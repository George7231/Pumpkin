use std::sync::Arc;

use pumpkin_data::{BlockState, BlockStateId};
use pumpkin_nbt::compound::NbtCompound;
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector2::Vector2;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::chunk::palette::BlockPalette;
use pumpkin_world::generation::structure::template::BlockPlacer;
use pumpkin_world::level::Level;

use crate::world::World;

impl World {
    pub fn clear_synced_block_events_in_box(&self, min: &BlockPos, max: &BlockPos) {
        let mut events = self
            .synced_block_event_queue
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        events.retain(|event| {
            let pos = event.pos;
            pos.0.x < min.0.x
                || pos.0.x >= max.0.x
                || pos.0.y < min.0.y
                || pos.0.y >= max.0.y
                || pos.0.z < min.0.z
                || pos.0.z >= max.0.z
        });
    }
}

pub struct WorldBlockPlacer<'a> {
    world: &'a Arc<World>,
    pub block_entity_nbts: Vec<NbtCompound>,
    pub changed_positions: Vec<(BlockPos, BlockStateId)>,
    post_processing_chunks: Vec<Vector2<i32>>,
}

impl<'a> WorldBlockPlacer<'a> {
    #[must_use]
    pub const fn new(world: &'a Arc<World>) -> Self {
        Self {
            world,
            block_entity_nbts: Vec::new(),
            changed_positions: Vec::new(),
            post_processing_chunks: Vec::new(),
        }
    }

    #[allow(clippy::unused_async)]
    pub fn finalize(&mut self) {
        for nbt in &self.block_entity_nbts {
            if let Some(block_entity) = crate::block::entities::block_entity_from_nbt(nbt) {
                self.world.add_block_entity(block_entity);
            }
        }
        for chunk_pos in std::mem::take(&mut self.post_processing_chunks) {
            self.world.run_post_processing(chunk_pos);
        }
        for (position, state_id) in &mut self.changed_positions {
            *state_id = self.world.get_block_state_id(position);
        }
    }
}

impl BlockPlacer for WorldBlockPlacer<'_> {
    fn get_block_state(&self, pos: &Vector3<i32>) -> BlockStateId {
        self.world
            .get_block_state_id(&BlockPos::new(pos.x, pos.y, pos.z))
    }

    fn set_block_state(&mut self, pos: &Vector3<i32>, state: &BlockState) {
        let block_pos = BlockPos::new(pos.x, pos.y, pos.z);
        Level::set_block_state(&self.world.level, &block_pos, state.id);
        self.changed_positions.push((block_pos, state.id));
    }

    fn add_block_entity(&mut self, nbt: NbtCompound) {
        self.block_entity_nbts.push(nbt);
    }

    fn mark_block_for_post_processing(&mut self, position: &BlockPos) {
        let chunk_pos = Vector2::new(position.0.x >> 4, position.0.z >> 4);
        let marked = self.world.level.read_chunk_sync(&chunk_pos, |chunk| {
            let local_y = position.0.y - chunk.section.min_y;
            if local_y < 0 || local_y as usize >= chunk.section.count * BlockPalette::SIZE {
                return false;
            }
            let packed = ((position.0.x & 15)
                | ((position.0.y & 15) << 4)
                | ((position.0.z & 15) << 8)) as i16;
            let mut sections = chunk
                .post_processing
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            sections.resize_with(chunk.section.count, Vec::new);
            sections[local_y as usize / BlockPalette::SIZE].push(packed);
            true
        });
        if marked == Some(true) && !self.post_processing_chunks.contains(&chunk_pos) {
            self.post_processing_chunks.push(chunk_pos);
        }
    }
}
