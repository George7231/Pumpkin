use crate::block::simple_waterlogged::SimpleWaterloggedBlock;
use crate::block::{
    BlockBehaviour, BucketPickup, CanPlaceLiquidArgs, LiquidBlockContainer, OnPlaceArgs,
    PickupFluidArgs, PlaceLiquidArgs,
};
use pumpkin_data::BlockStateId;
use pumpkin_data::block_properties::MangroveRootsLikeProperties as BarrierLikeProperties;
use pumpkin_data::fluid::Fluid;
use pumpkin_data::item::Item;
use pumpkin_macros::pumpkin_block;
use pumpkin_util::GameMode;

#[pumpkin_block("minecraft:barrier")]
pub struct BarrierBlock;

impl LiquidBlockContainer for BarrierBlock {
    fn can_place_liquid(&self, args: CanPlaceLiquidArgs<'_>) -> bool {
        args.fluid == &Fluid::WATER
            && args
                .player
                .is_some_and(|player| player.gamemode.load() == GameMode::Creative)
    }

    fn place_liquid(&self, args: PlaceLiquidArgs<'_>) -> bool {
        SimpleWaterloggedBlock.place_liquid(args)
    }
}

impl BucketPickup for BarrierBlock {
    fn pickup_fluid(&self, args: PickupFluidArgs<'_>) -> Option<&'static Item> {
        args.player
            .filter(|player| player.gamemode.load() == GameMode::Creative)?;
        SimpleWaterloggedBlock.pickup_fluid(args)
    }
}

impl BlockBehaviour for BarrierBlock {
    fn as_liquid_block_container(&self) -> Option<&dyn LiquidBlockContainer> {
        Some(self)
    }

    fn as_bucket_pickup(&self) -> Option<&dyn BucketPickup> {
        Some(self)
    }

    fn on_place(&self, args: OnPlaceArgs<'_>) -> BlockStateId {
        let mut props = BarrierLikeProperties::default(args.block);
        props.waterlogged = args.replacing.water_source();
        props.to_state_id(args.block)
    }
}
