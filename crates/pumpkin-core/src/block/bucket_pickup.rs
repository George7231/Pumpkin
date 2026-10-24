use std::sync::Arc;

use pumpkin_data::{Block, BlockStateId, item::Item};
use pumpkin_util::math::position::BlockPos;

use crate::{entity::player::Player, world::World};

/// A block whose contents can be collected in a bucket.
pub trait BucketPickup: Send + Sync {
    fn pickup_fluid(&self, args: PickupFluidArgs<'_>) -> Option<&'static Item>;
}

#[derive(Clone, Copy)]
pub struct PickupFluidArgs<'a> {
    pub player: Option<&'a Player>,
    pub world: &'a Arc<World>,
    pub block: &'a Block,
    pub state_id: BlockStateId,
    pub position: &'a BlockPos,
}
