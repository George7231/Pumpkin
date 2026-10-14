use pumpkin_data::fluid::Fluid;
use pumpkin_data::tag::{self, Taggable};
use pumpkin_data::{Block, BlockDirection, BlockStateId};
use pumpkin_macros::pumpkin_block;
use pumpkin_util::math::position::BlockPos;
use pumpkin_world::tick::TickPriority;

use super::bubble_column::BubbleColumnBlock;
use crate::block::{
    BlockBehaviour, GetStateForNeighborUpdateArgs, OnScheduledTickArgs, PlacedArgs,
};
use crate::world::World;

const SCHEDULED_TICK_DELAY: u8 = 20;

#[pumpkin_block("minecraft:water")]
pub struct LiquidBlock;

impl LiquidBlock {
    fn should_bubble_column_occupy(state: BlockStateId) -> bool {
        state == Block::WATER.default_state.id
            && Fluid::WATER.has_tag(&tag::Fluid::MINECRAFT_BUBBLE_COLUMN_CAN_OCCUPY)
    }

    fn try_schedule_bubble_block_column(world: &World, position: &BlockPos, below: &Block) {
        if below.has_tag(&tag::Block::MINECRAFT_ENABLES_BUBBLE_COLUMN_DRAG_DOWN)
            || below.has_tag(&tag::Block::MINECRAFT_ENABLES_BUBBLE_COLUMN_PUSH_UP)
        {
            world.schedule_block_tick(
                &Block::WATER,
                *position,
                SCHEDULED_TICK_DELAY,
                TickPriority::Normal,
            );
        }
    }
}

impl BlockBehaviour for LiquidBlock {
    fn placed(&self, args: PlacedArgs<'_>) {
        if Self::should_bubble_column_occupy(args.state_id) {
            Self::try_schedule_bubble_block_column(
                args.world,
                args.position,
                args.world.get_block(&args.position.down()),
            );
        }
    }

    fn get_state_for_neighbor_update(
        &self,
        args: GetStateForNeighborUpdateArgs<'_>,
    ) -> BlockStateId {
        if args.direction == BlockDirection::Down
            && Self::should_bubble_column_occupy(args.state_id)
        {
            Self::try_schedule_bubble_block_column(
                args.world,
                args.position,
                args.neighbor_state_id.to_block(),
            );
        }
        args.state_id
    }

    fn on_scheduled_tick(&self, args: OnScheduledTickArgs<'_>) {
        if Self::should_bubble_column_occupy(args.world.get_block_state_id(args.position)) {
            BubbleColumnBlock::update_column(args.world, *args.position);
        }
    }
}
