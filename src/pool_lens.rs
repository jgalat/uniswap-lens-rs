//! ## Pool Lens
//!
//! The pool lens module provides functions to fetch pool details using ephemeral contracts.

use crate::{
    amm::Dex, bindings::{
        ephemeralgetpopulatedticksinrange::{
            EphemeralGetPopulatedTicksInRange::{self, getPopulatedTicksInRangeCall},
            PoolUtils::PopulatedTick,
        }, ephemeralgetpopulatedticksinrangev4::{EphemeralGetPopulatedTicksInRangeV4::{self, PoolKey, getPopulatedTicksInRangeCall as getPopulatedTicksInRangeV4Call}, PoolUtilsV4::PopulatedTick as PopulatedTickV4},
        ephemeralpoolpositions::{EphemeralPoolPositions, PoolUtils::PositionKey}, 
        ephemeralpoolslots::{
            EphemeralPoolSlots::{self, getSlotsCall}, PoolUtils::Slot,
        }, ephemeralpooltickbitmap::EphemeralPoolTickBitmap, ephemeralpoolticks::EphemeralPoolTicks
    }, call_ephemeral_contract, error::Error
};
use alloc::vec::Vec;
use alloy::{
    contract::Error as ContractError,
    eips::BlockId,
    network::Network,
    primitives::{aliases::I24, Address},
    providers::Provider,
    sol_types::SolCall,
    transports::TransportError,
};

/// Get the populated ticks in a tick range.
///
/// ## Arguments
///
/// * `dex`: Dex
/// * `pool`: The address of a pool
/// * `tick_lower`: The lower tick boundary
/// * `tick_upper`: The upper tick boundary
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of populated ticks within the range
#[inline]
pub async fn get_populated_ticks_in_range<N, P>(
    amm: Dex,
    pool: Address,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<PopulatedTick>, Error>
where
    N: Network,
    P: Provider<N>,
{
    let deploy_builder =
        EphemeralGetPopulatedTicksInRange::deploy_builder(provider, amm.into(), pool, tick_lower, tick_upper);
    match call_ephemeral_contract!(deploy_builder, getPopulatedTicksInRangeCall, block_id) {
        Ok(populated_ticks) => Ok(
            populated_ticks
                .into_iter()
                .filter(|PopulatedTick { tick, .. }| *tick >= tick_lower && *tick <= tick_upper)
                .collect()
        ),
        Err(err) => Err(err),
    }
}

#[inline]
pub async fn get_populated_ticks_in_range_v4<N, P>(
    pool_manager: Address,
    pool_key: PoolKey,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<PopulatedTickV4>, Error>
where
    N: Network,
    P: Provider<N>,
{
    let deploy_builder =
        EphemeralGetPopulatedTicksInRangeV4::deploy_builder(provider, pool_manager, pool_key, tick_lower, tick_upper);
    match call_ephemeral_contract!(deploy_builder, getPopulatedTicksInRangeV4Call, block_id) {
        Ok(populated_ticks) => Ok(
            populated_ticks
                .into_iter()
                .filter(|PopulatedTickV4 { tick, .. }| *tick >= tick_lower && *tick <= tick_upper)
                .collect()
        ),
        Err(err) => Err(err),
    }
}

/// Call an ephemeral contract and return the decoded storage slots
macro_rules! get_pool_storage {
    ($deploy_builder:expr, $block_id:expr) => {
        call_ephemeral_contract!($deploy_builder, getSlotsCall, $block_id)
    };
}

/// Get the static storage slots of a pool.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_static_slots<N, P>(
    amm: Dex,
    pool: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    if matches!(amm, Dex::SlipStream | Dex::Algebra) {
        return Err(Error::DexNotSupported);
    }
    get_pool_storage!(EphemeralPoolSlots::deploy_builder(provider, amm.into(), pool), block_id)
}

/// Get the storage slots in the `ticks` mapping between `tick_lower` and `tick_upper`.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `tick_lower`: The lower tick boundary
/// * `tick_upper`: The upper tick boundary
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_ticks_slots<N, P>(
    amm: Dex,
    pool: Address,
    tick_lower: I24,
    tick_upper: I24,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    if matches!(amm, Dex::SlipStream | Dex::Algebra) {
        return Err(Error::DexNotSupported);
    }
    get_pool_storage!(
        EphemeralPoolTicks::deploy_builder(provider, amm.into(), pool, tick_lower, tick_upper),
        block_id
    )
}

/// Get the storage slots in the `tickBitmap` mapping.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_tick_bitmap_slots<N, P>(
    amm: Dex,
    pool: Address,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    if matches!(amm, Dex::SlipStream | Dex::Algebra) {
        return Err(Error::DexNotSupported);
    }
    get_pool_storage!(
        EphemeralPoolTickBitmap::deploy_builder(provider, amm.into(), pool),
        block_id
    )
}

/// Get the storage slots in the `positions` mapping.
///
/// ## Arguments
///
/// * `pool`: The address of a V3 pool
/// * `positions`: A vector of position keys
/// * `provider`: The alloy provider
/// * `block_id`: Optional block number to query
///
/// ## Returns
///
/// A vector of slots containing the storage data
#[inline]
pub async fn get_positions_slots<N, P>(
    amm: Dex,
    pool: Address,
    positions: Vec<PositionKey>,
    provider: P,
    block_id: Option<BlockId>,
) -> Result<Vec<Slot>, Error>
where
    N: Network,
    P: Provider<N>,
{
    get_pool_storage!(
        EphemeralPoolPositions::deploy_builder(provider, amm.into(), pool, positions),
        block_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bindings::iuniswapv3pool::{IUniswapV3Pool, IUniswapV3Pool::Mint},
        tests::*,
    };
    use alloy::{
        primitives::{address, U256}, providers::MulticallBuilder, rpc::types::Filter, sol_types::SolEvent,
    };
    use futures::future::join_all;

    const POOL_ADDRESS: Address = address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");

    #[tokio::test]
    async fn test_get_populated_ticks_in_range() {
        let provider = PROVIDER.clone();
        let pool = IUniswapV3Pool::new(POOL_ADDRESS, provider.clone());
        let tick_current = pool.slot0().block(BLOCK_NUMBER).call().await.unwrap().tick;
        let tick_spacing = pool.tickSpacing().block(BLOCK_NUMBER).call().await.unwrap();
        let ticks = get_populated_ticks_in_range(
            Dex::UniswapV3,
            POOL_ADDRESS,
            tick_current,
            tick_current + (tick_spacing << 8),
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        assert!(!ticks.is_empty());

        let mut multicall = MulticallBuilder::new_dynamic(provider.clone());
        for PopulatedTick { tick, .. } in ticks.iter() {
            multicall = multicall.add_dynamic(pool.ticks(*tick));
        }
        let alt_ticks = multicall.block(BLOCK_NUMBER).aggregate().await.unwrap();

        for (i, tick) in ticks.into_iter().enumerate() {
            let tick_info = &alt_ticks[i];
            assert_eq!(tick.liquidityGross, tick_info.liquidityGross);
            assert_eq!(tick.liquidityNet, tick_info.liquidityNet);
            assert_eq!(tick.feeGrowthOutside0X128, tick_info.feeGrowthOutside0X128);
            assert_eq!(tick.feeGrowthOutside1X128, tick_info.feeGrowthOutside1X128);
        }
    }

    #[tokio::test]
    async fn test_get_populated_ticks_in_range_pancakeswap_v3() {
        // PancakeSwapV3 WETH/USDC on Ethereum
        let pancake_pool = address!("6CA298D2983aB03Aa1da7679389D955A4eFEE15C");
        let provider = PROVIDER.clone();
        let pool = IUniswapV3Pool::new(pancake_pool, provider.clone());
        let tick_current = pool.slot0().block(BLOCK_NUMBER).call().await.unwrap().tick;
        let tick_spacing = pool.tickSpacing().block(BLOCK_NUMBER).call().await.unwrap();
        let ticks = get_populated_ticks_in_range(
            Dex::PancakeSwapV3,
            pancake_pool,
            tick_current,
            tick_current + (tick_spacing << 8),
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        assert!(!ticks.is_empty());

        let mut multicall = MulticallBuilder::new_dynamic(provider.clone());
        for PopulatedTick { tick, .. } in ticks.iter() {
            multicall = multicall.add_dynamic(pool.ticks(*tick));
        }
        let alt_ticks = multicall.block(BLOCK_NUMBER).aggregate().await.unwrap();

        for (i, tick) in ticks.into_iter().enumerate() {
            let tick_info = &alt_ticks[i];
            assert_eq!(tick.liquidityGross, tick_info.liquidityGross);
            assert_eq!(tick.liquidityNet, tick_info.liquidityNet);
            assert_eq!(tick.feeGrowthOutside0X128, tick_info.feeGrowthOutside0X128);
            assert_eq!(tick.feeGrowthOutside1X128, tick_info.feeGrowthOutside1X128);
        }
    }

    #[tokio::test]
    async fn test_get_populated_ticks_in_range_slipstream() {
        use crate::tests::{BASE_BLOCK_NUMBER, BASE_PROVIDER};
        use alloy::sol;

        sol! {
            #[sol(rpc)]
            interface ISlipStreamCLPool {
                function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, bool unlocked);
                function tickSpacing() external view returns (int24);
                function ticks(int24 tick) external view returns (uint128 liquidityGross, int128 liquidityNet, int128 stakedLiquidityNet, uint256 feeGrowthOutside0X128, uint256 feeGrowthOutside1X128, uint256 rewardGrowthOutsideX128, int56 tickCumulativeOutside, uint160 secondsPerLiquidityOutsideX128, uint32 secondsOutside, bool initialized);
            }
        }

        // Aerodrome WETH/USDC on Base
        let aero_pool = address!("b2cc224c1c9feE385f8ad6a55b4d94E92359DC59");
        let provider = BASE_PROVIDER.clone();
        let pool = ISlipStreamCLPool::new(aero_pool, provider.clone());
        let slot0 = pool.slot0().block(BASE_BLOCK_NUMBER).call().await.unwrap();
        let tick_spacing = pool.tickSpacing().block(BASE_BLOCK_NUMBER).call().await.unwrap();
        let ticks = get_populated_ticks_in_range(
            Dex::SlipStream,
            aero_pool,
            slot0.tick,
            slot0.tick + (tick_spacing << 8),
            provider.clone(),
            Some(BASE_BLOCK_NUMBER),
        )
        .await
        .unwrap();
        assert!(!ticks.is_empty());

        let mut multicall = MulticallBuilder::new_dynamic(provider.clone());
        for PopulatedTick { tick, .. } in ticks.iter() {
            multicall = multicall.add_dynamic(pool.ticks(*tick));
        }
        let alt_ticks = multicall.block(BASE_BLOCK_NUMBER).aggregate().await.unwrap();

        for (i, tick) in ticks.into_iter().enumerate() {
            let tick_info = &alt_ticks[i];
            assert_eq!(tick.liquidityGross, tick_info.liquidityGross);
            assert_eq!(tick.liquidityNet, tick_info.liquidityNet);
            assert_eq!(tick.feeGrowthOutside0X128, tick_info.feeGrowthOutside0X128);
            assert_eq!(tick.feeGrowthOutside1X128, tick_info.feeGrowthOutside1X128);
        }
    }

    #[tokio::test]
    async fn test_get_populated_ticks_in_range_algebra() {
        use crate::tests::{BASE_BLOCK_NUMBER, BASE_PROVIDER};
        use alloy::sol;

        sol! {
            #[sol(rpc)]
            interface IAlgebraPool {
                function globalState() external view returns (uint160 price, int24 tick, uint16 fee, uint16 timepointIndex, uint8 communityFeeToken0, bool unlocked);
                function tickSpacing() external view returns (int24);
                function ticks(int24 tick) external view returns (uint256 liquidityTotal, int128 liquidityDelta, int24 prevTick, int24 nextTick, uint256 outerFeeGrowth0Token, uint256 outerFeeGrowth1Token);
            }
        }

        // QuickSwap WETH/USDC on Base
        let algebra_pool = address!("5a9Ad2BB92B0B3E5C571FDD5125114E04E02be1a");
        let provider = BASE_PROVIDER.clone();
        let pool = IAlgebraPool::new(algebra_pool, provider.clone());
        let state = pool.globalState().block(BASE_BLOCK_NUMBER).call().await.unwrap();
        let tick_spacing = pool.tickSpacing().block(BASE_BLOCK_NUMBER).call().await.unwrap();
        let ticks = get_populated_ticks_in_range(
            Dex::Algebra,
            algebra_pool,
            state.tick,
            state.tick + (tick_spacing << 8),
            provider.clone(),
            Some(BASE_BLOCK_NUMBER),
        )
        .await
        .unwrap();
        assert!(!ticks.is_empty());

        let mut multicall = MulticallBuilder::new_dynamic(provider.clone());
        for PopulatedTick { tick, .. } in ticks.iter() {
            multicall = multicall.add_dynamic(pool.ticks(*tick));
        }
        let alt_ticks = multicall.block(BASE_BLOCK_NUMBER).aggregate().await.unwrap();

        for (i, tick) in ticks.into_iter().enumerate() {
            let tick_info = &alt_ticks[i];
            assert_eq!(U256::from(tick.liquidityGross), tick_info.liquidityTotal);
            assert_eq!(tick.liquidityNet, tick_info.liquidityDelta);
            assert_eq!(tick.feeGrowthOutside0X128, tick_info.outerFeeGrowth0Token);
            assert_eq!(tick.feeGrowthOutside1X128, tick_info.outerFeeGrowth1Token);
        }
    }

    #[tokio::test]
    async fn test_get_populated_ticks_in_range_v4() {
        use alloy::primitives::{aliases::I24, Uint};

        // Uniswap V4 PoolManager on Ethereum
        let pool_manager = address!("000000000004444c5dc75cB358380D2e3dE08A90");
        // ETH/USDC pool: fee=3000, tickSpacing=60, hooks=0x0
        let pool_key = PoolKey {
            currency0: Address::ZERO,
            currency1: address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"),
            fee: Uint::from(3000),
            tickSpacing: I24::unchecked_from(60),
            hooks: Address::ZERO,
        };
        // tick at initialization was ~-196257, query a range around it
        let tick_lower = I24::unchecked_from(-199_980);
        let tick_upper = I24::unchecked_from(-193_980);
        let ticks = get_populated_ticks_in_range_v4(
            pool_manager,
            pool_key,
            tick_lower,
            tick_upper,
            PROVIDER.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        assert!(!ticks.is_empty());
        for tick in &ticks {
            assert!(tick.tick >= tick_lower && tick.tick <= tick_upper);
            assert!(tick.liquidityGross > 0);
        }
    }

    #[tokio::test]
    async fn test_storage_lens_dex_not_supported() {
        use crate::tests::{BASE_BLOCK_NUMBER, BASE_PROVIDER};

        let pool = address!("0000000000000000000000000000000000000001");
        for dex in [Dex::SlipStream, Dex::Algebra] {
            assert!(matches!(
                get_static_slots(dex, pool, BASE_PROVIDER.clone(), Some(BASE_BLOCK_NUMBER)).await,
                Err(Error::DexNotSupported)
            ));
            assert!(matches!(
                get_ticks_slots(dex, pool, I24::ZERO, I24::ZERO, BASE_PROVIDER.clone(), Some(BASE_BLOCK_NUMBER)).await,
                Err(Error::DexNotSupported)
            ));
            assert!(matches!(
                get_tick_bitmap_slots(dex, pool, BASE_PROVIDER.clone(), Some(BASE_BLOCK_NUMBER)).await,
                Err(Error::DexNotSupported)
            ));
        }
    }

    async fn verify_slots<N, P>(slots: Vec<Slot>, provider: P)
    where
        N: Network,
        P: Provider<N>,
    {
        assert!(!slots.is_empty());
        let provider = provider.root();
        let futures = slots[0..4].iter().map(|slot| async move {
            let data = provider
                .get_storage_at(POOL_ADDRESS, slot.slot)
                .block_id(BLOCK_NUMBER)
                .await
                .unwrap();
            assert!(slot.data.eq(&data));
        });
        join_all(futures).await;
    }

    #[tokio::test]
    async fn test_get_static_slots() {
        let provider = PROVIDER.clone();
        let slots = get_static_slots(Dex::UniswapV3,POOL_ADDRESS, provider.clone(), Some(BLOCK_NUMBER))
            .await
            .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_ticks_slots() {
        let provider = PROVIDER.clone();
        let pool = IUniswapV3Pool::new(POOL_ADDRESS, provider.clone());
        let tick_current = pool.slot0().block(BLOCK_NUMBER).call().await.unwrap().tick;
        let slots = get_ticks_slots(
            Dex::UniswapV3,
            POOL_ADDRESS,
            tick_current,
            tick_current,
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_tick_bitmap_slots() {
        let provider = PROVIDER.clone();
        let slots = get_tick_bitmap_slots(Dex::UniswapV3, POOL_ADDRESS, provider.clone(), Some(BLOCK_NUMBER))
            .await
            .unwrap();
        verify_slots(slots, provider).await;
    }

    #[tokio::test]
    async fn test_get_positions_slots() {
        let provider = PROVIDER.clone();
        // create a filter to get the mint events
        let filter = Filter::new()
            .from_block(BLOCK_NUMBER.as_u64().unwrap() - 499)
            .to_block(BLOCK_NUMBER.as_u64().unwrap())
            .event_signature(<Mint as SolEvent>::SIGNATURE_HASH);
        let logs = provider.get_logs(&filter).await.unwrap();
        // decode the logs into position keys
        let positions: Vec<_> = logs
            .iter()
            .map(|log| <Mint as SolEvent>::decode_log_data(log.data()).unwrap())
            .map(
                |Mint {
                     owner,
                     tickLower,
                     tickUpper,
                     ..
                 }| PositionKey {
                    owner,
                    tickLower,
                    tickUpper,
                },
            )
            .collect();
        assert!(!positions.is_empty());
        let slots = get_positions_slots(
            Dex::UniswapV3,
            POOL_ADDRESS,
            positions,
            provider.clone(),
            Some(BLOCK_NUMBER),
        )
        .await
        .unwrap();
        verify_slots(slots, provider).await;
    }
}
