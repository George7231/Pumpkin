use std::sync::Arc;

use pumpkin_data::{Block, BlockStateId, fluid::Fluid};
use pumpkin_util::math::position::BlockPos;

use crate::{entity::player::Player, world::World};

/// A block that can contain liquid without being replaced.
pub trait LiquidBlockContainer: Send + Sync {
    /// Checks whether the block accepts the fluid without changing the world.
    fn can_place_liquid(&self, args: CanPlaceLiquidArgs<'_>) -> bool;

    /// Attempts to fill the block; bucket use may succeed even when this returns false.
    fn place_liquid(&self, args: PlaceLiquidArgs<'_>) -> bool;
}

#[derive(Clone, Copy)]
pub struct CanPlaceLiquidArgs<'a> {
    pub player: Option<&'a Player>,
    pub state_id: BlockStateId,
    pub fluid: &'a Fluid,
}

#[derive(Clone, Copy)]
pub struct PlaceLiquidArgs<'a> {
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub position: &'a BlockPos,
    pub fluid: &'a Fluid,
}
