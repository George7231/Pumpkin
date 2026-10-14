use std::sync::Arc;

use pumpkin_data::block_properties::BubbleColumnLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockId, BlockStateId};
use pumpkin_util::math::position::BlockPos;
use pumpkin_util::math::vector3::Vector3;
use pumpkin_world::tick::TickPriority;
use pumpkin_world::world::BlockFlags;

use crate::block::{
    BlockBehaviour, BlockMetadata, GetStateForNeighborUpdateArgs, OnEntityCollisionArgs,
    OnScheduledTickArgs,
};
use crate::world::World;

const SCHEDULED_TICK_DELAY: u8 = 5;

const UPWARD_ACCELERATION: f64 = 0.06;
const UPWARD_MAX_SPEED: f64 = 0.7;
const SURFACE_UPWARD_ACCELERATION: f64 = 0.1;
const SURFACE_UPWARD_MAX_SPEED: f64 = 1.8;
const DOWNWARD_ACCELERATION: f64 = -0.03;
const DOWNWARD_MIN_SPEED: f64 = -0.3;

pub struct BubbleColumnBlock;

impl BlockMetadata for BubbleColumnBlock {
    fn ids() -> Box<[BlockId]> {
        [BlockId::BUBBLE_COLUMN].into()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BubbleColumnKind {
    Upward,
    Downward,
}

fn bubble_column_state(kind: BubbleColumnKind) -> BlockStateId {
    BubbleColumnLikeProperties {
        r#drag: matches!(kind, BubbleColumnKind::Downward),
    }
    .to_state_id(&Block::BUBBLE_COLUMN)
}

fn kind_from_support(block: &Block) -> Option<BubbleColumnKind> {
    if block.has_tag(&tag::Block::MINECRAFT_ENABLES_BUBBLE_COLUMN_PUSH_UP) {
        Some(BubbleColumnKind::Upward)
    } else if block.has_tag(&tag::Block::MINECRAFT_ENABLES_BUBBLE_COLUMN_DRAG_DOWN) {
        Some(BubbleColumnKind::Downward)
    } else {
        None
    }
}

fn kind_from_state(state: BlockStateId) -> BubbleColumnKind {
    let props = BubbleColumnLikeProperties::from_state_id(state);
    if props.r#drag {
        BubbleColumnKind::Downward
    } else {
        BubbleColumnKind::Upward
    }
}

fn is_still_water(state: BlockStateId) -> bool {
    state.to_block_id() == Block::BUBBLE_COLUMN || state == Block::WATER.default_state.id
}

fn get_bubble_state(state: BlockStateId) -> BlockStateId {
    if state.to_block_id() == Block::BUBBLE_COLUMN {
        state
    } else {
        kind_from_support(state.to_block())
            .map_or(Block::WATER.default_state.id, bubble_column_state)
    }
}

fn bubble_column_vertical_velocity(
    current_y: f64,
    kind: BubbleColumnKind,
    at_surface: bool,
) -> f64 {
    match (kind, at_surface) {
        (BubbleColumnKind::Upward, true) => {
            (current_y + SURFACE_UPWARD_ACCELERATION).min(SURFACE_UPWARD_MAX_SPEED)
        }
        (BubbleColumnKind::Upward, false) => {
            (current_y + UPWARD_ACCELERATION).min(UPWARD_MAX_SPEED)
        }
        (BubbleColumnKind::Downward, _) => {
            (current_y + DOWNWARD_ACCELERATION).max(DOWNWARD_MIN_SPEED)
        }
    }
}

fn bubble_column_velocity(
    current: Vector3<f64>,
    kind: BubbleColumnKind,
    at_surface: bool,
) -> Vector3<f64> {
    Vector3::new(
        current.x,
        bubble_column_vertical_velocity(current.y, kind, at_surface),
        current.z,
    )
}

impl BubbleColumnBlock {
    pub(crate) fn update_column(world: &Arc<World>, position: BlockPos) {
        let state = world.get_block_state_id(&position);
        if !is_still_water(state) {
            return;
        }
        let new_state = get_bubble_state(world.get_block_state_id(&position.down()));
        world.set_block_state(&position, new_state, BlockFlags::NOTIFY_LISTENERS);

        let mut position = position.up();
        while is_still_water(world.get_block_state_id(&position)) {
            if world.set_block_state(&position, new_state, BlockFlags::NOTIFY_LISTENERS)
                == new_state
            {
                break;
            }
            position = position.up();
        }
    }
}

impl BlockBehaviour for BubbleColumnBlock {
    fn on_entity_collision(&self, args: OnEntityCollisionArgs<'_>) {
        {
            if args.block != &Block::BUBBLE_COLUMN {
                return;
            }

            let kind = kind_from_state(args.state.id);
            let at_surface = args.world.get_block(&args.position.up()) == &Block::AIR;
            let entity = args.entity.get_entity();
            entity.velocity.store(bubble_column_velocity(
                entity.velocity.load(),
                kind,
                at_surface,
            ));

            if let Some(player) = args.entity.get_player() {
                player.breath_manager.reset(player);
            }
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        args.world.schedule_fluid_tick(
            &Fluid::WATER,
            *args.position,
            Fluid::WATER.flow_speed as u8,
            TickPriority::Normal,
        );
        let below_state = args.world.get_block_state_id(&args.position.down());
        if get_bubble_state(below_state) == Block::WATER.default_state.id
            || args.direction == BlockDirection::Down
            || (args.direction == BlockDirection::Up
                && args.neighbor_state_id.to_block_id() != Block::BUBBLE_COLUMN
                && is_still_water(args.neighbor_state_id))
        {
            args.world.schedule_block_tick(
                args.block,
                *args.position,
                SCHEDULED_TICK_DELAY,
                TickPriority::Normal,
            );
        }
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        Self::update_column(args.world, *args.position);
    }
}

#[cfg(test)]
mod tests {
    use pumpkin_data::block_properties::WaterLikeProperties;

    use super::*;

    #[test]
    fn still_water_excludes_flowing_water_and_water_containing_blocks() {
        assert!(is_still_water(Block::WATER.default_state.id));
        assert!(is_still_water(Block::BUBBLE_COLUMN.default_state.id));
        for level in 1..=15 {
            assert!(!is_still_water(
                WaterLikeProperties { level }.to_state_id(&Block::WATER)
            ));
        }
        assert!(!is_still_water(Block::KELP.default_state.id));
        assert!(!is_still_water(Block::AIR.default_state.id));
    }

    #[test]
    fn bubble_state_inherits_direction_and_restores_water_without_support() {
        for kind in [BubbleColumnKind::Upward, BubbleColumnKind::Downward] {
            let state = bubble_column_state(kind);
            assert_eq!(get_bubble_state(state), state);
        }
        assert_eq!(
            get_bubble_state(Block::SOUL_SAND.default_state.id),
            bubble_column_state(BubbleColumnKind::Upward)
        );
        assert_eq!(
            get_bubble_state(Block::MAGMA_BLOCK.default_state.id),
            bubble_column_state(BubbleColumnKind::Downward)
        );
        assert_eq!(
            get_bubble_state(Block::STONE.default_state.id),
            Block::WATER.default_state.id
        );
    }
}
