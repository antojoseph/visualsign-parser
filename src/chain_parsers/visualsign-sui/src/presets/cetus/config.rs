#![allow(dead_code)]

crate::chain_config! {
  config CETUS_CONFIG as Config;

  cetus_mainnet => {
      package_id => 0xb2db7142fa83210a7d78d9c12ac49c043b3cbbd482224fea6e3da00aa5a5ae2d,
      modules as CetusModules: {
        pool_script_v2 as PoolScriptV2 => PoolScriptV2Functions: {
          swap_a2b as SwapA2B => SwapA2BIndexes(
                by_amount_in as ByAmountIn: bool => 4 => get_by_amount_in,
                amount as Amount: u64 => 5 => get_amount,
                amount_limit as AmountLimit: u64 => 6 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
          ),
          swap_b2a as SwapB2A => SwapB2AIndexes(
                by_amount_in as ByAmountIn: bool => 4 => get_by_amount_in,
                amount as Amount: u64 => 5 => get_amount,
                amount_limit as AmountLimit: u64 => 6 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
          ),
          swap_a2b_with_partner as SwapA2BWithPartner => SwapA2BWithPartnerIndexes(
                by_amount_in as ByAmountIn: bool => 5 => get_by_amount_in,
                amount as Amount: u64 => 6 => get_amount,
                amount_limit as AmountLimit: u64 => 7 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 8 => get_sqrt_price_limit,
          ),
          swap_b2a_with_partner as SwapB2AWithPartner => SwapB2AWithPartnerIndexes(
                by_amount_in as ByAmountIn: bool => 5 => get_by_amount_in,
                amount as Amount: u64 => 6 => get_amount,
                amount_limit as AmountLimit: u64 => 7 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 8 => get_sqrt_price_limit,
          ),
          add_liquidity_by_fix_coin as AddLiquidityByFixCoin => AddLiquidityByFixCoinIndexes(
                amount_a as AmountA: u64 => 5 => get_amount_a,
                amount_b as AmountB: u64 => 6 => get_amount_b,
                is_fix_a as IsFixA: bool => 7 => get_is_fix_a,
          ),
          open_position_with_liquidity_by_fix_coin as OpenPositionWithLiquidityByFixCoin => OpenPositionWithLiquidityByFixCoinIndexes(
                amount_a as AmountA: u64 => 6 => get_amount_a,
                amount_b as AmountB: u64 => 7 => get_amount_b,
                is_fix_a as IsFixA: bool => 8 => get_is_fix_a,
          ),
          open_position_with_liquidity_with_all as OpenPositionWithLiquidityWithAll => OpenPositionWithLiquidityWithAllIndexes(
                amount_a as AmountA: u64 => 6 => get_amount_a,
                amount_b as AmountB: u64 => 7 => get_amount_b,
                is_fix_a as IsFixA: bool => 8 => get_is_fix_a,
          ),
          remove_liquidity as RemoveLiquidity => RemoveLiquidityIndexes(
                liquidity as Liquidity: u128 => 3 => get_liquidity,
                min_amount_a as MinAmountA: u64 => 4 => get_min_amount_a,
                min_amount_b as MinAmountB: u64 => 5 => get_min_amount_b,
          ),
          close_position as ClosePosition => ClosePositionIndexes(
                min_amount_a as MinAmountA: u64 => 3 => get_min_amount_a,
                min_amount_b as MinAmountB: u64 => 4 => get_min_amount_b,
          ),
          collect_fee as CollectFee => CollectFeeIndexes(),
          collect_reward as CollectReward => CollectRewardIndexes(),
        },
        router as Router => RouterFunctions: {
          swap as Swap => RouterSwapIndexes(
                is_a2b as IsA2B: bool => 4 => get_is_a2b,
                by_amount_in as ByAmountIn: bool => 5 => get_by_amount_in,
                amount as Amount: u64 => 6 => get_amount,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
                use_all_coin as UseAllCoin: bool => 8 => get_use_all_coin,
          ),
          check_coin_threshold as CheckCoinThreshold => RouterCheckCoinThresholdIndexes(
                threshold as Threshold: u64 => 1 => get_threshold,
          ),
        },
        pool_script as PoolScript => PoolScriptFunctions: {
          swap_a2b as SwapA2B => PoolScriptSwapA2BIndexes(
                by_amount_in as ByAmountIn: bool => 3 => get_by_amount_in,
                amount as Amount: u64 => 4 => get_amount,
                amount_limit as AmountLimit: u64 => 5 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 6 => get_sqrt_price_limit,
          ),
          swap_b2a as SwapB2A => PoolScriptSwapB2AIndexes(
                by_amount_in as ByAmountIn: bool => 3 => get_by_amount_in,
                amount as Amount: u64 => 4 => get_amount,
                amount_limit as AmountLimit: u64 => 5 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 6 => get_sqrt_price_limit,
          ),
          swap_a2b_with_partner as SwapA2BWithPartner => PoolScriptSwapA2BWithPartnerIndexes(
                by_amount_in as ByAmountIn: bool => 4 => get_by_amount_in,
                amount as Amount: u64 => 5 => get_amount,
                amount_limit as AmountLimit: u64 => 6 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
          ),
          swap_b2a_with_partner as SwapB2AWithPartner => PoolScriptSwapB2AWithPartnerIndexes(
                by_amount_in as ByAmountIn: bool => 4 => get_by_amount_in,
                amount as Amount: u64 => 5 => get_amount,
                amount_limit as AmountLimit: u64 => 6 => get_amount_limit,
                sqrt_price_limit as SqrtPriceLimit: u128 => 7 => get_sqrt_price_limit,
          ),
          collect_reward as CollectReward => PoolScriptCollectRewardIndexes(),
          collect_fee as CollectFee => PoolScriptCollectFeeIndexes(),
          close_position as ClosePosition => PoolScriptClosePositionIndexes(
                min_amount_a as MinAmountA: u64 => 3 => get_min_amount_a,
                min_amount_b as MinAmountB: u64 => 4 => get_min_amount_b,
          ),
          remove_liquidity as RemoveLiquidity => PoolScriptRemoveLiquidityIndexes(
                liquidity as Liquidity: u128 => 3 => get_liquidity,
                min_amount_a as MinAmountA: u64 => 4 => get_min_amount_a,
                min_amount_b as MinAmountB: u64 => 5 => get_min_amount_b,
          ),
          add_liquidity_fix_coin_only_a as AddLiquidityFixCoinOnlyA => PoolScriptAddLiquidityFixCoinOnlyAIndexes(
                amount_a as AmountA: u64 => 4 => get_amount_a,
          ),
          add_liquidity_fix_coin_only_b as AddLiquidityFixCoinOnlyB => PoolScriptAddLiquidityFixCoinOnlyBIndexes(
                amount_b as AmountB: u64 => 4 => get_amount_b,
          ),
          add_liquidity_fix_coin_with_all as AddLiquidityFixCoinWithAll => PoolScriptAddLiquidityFixCoinWithAllIndexes(
                amount_a as AmountA: u64 => 5 => get_amount_a,
                amount_b as AmountB: u64 => 6 => get_amount_b,
                is_fix_a as IsFixA: bool => 7 => get_is_fix_a,
          ),
          open_position_with_liquidity_only_a as OpenPositionWithLiquidityOnlyA => PoolScriptOpenPositionWithLiquidityOnlyAIndexes(
                amount_a as AmountA: u64 => 5 => get_amount_a,
          ),
          open_position_with_liquidity_only_b as OpenPositionWithLiquidityOnlyB => PoolScriptOpenPositionWithLiquidityOnlyBIndexes(
                amount_b as AmountB: u64 => 5 => get_amount_b,
          ),
          open_position_with_liquidity_with_all as OpenPositionWithLiquidityWithAll => PoolScriptOpenPositionWithLiquidityWithAllIndexes(
                amount_a as AmountA: u64 => 6 => get_amount_a,
                amount_b as AmountB: u64 => 7 => get_amount_b,
                is_fix_a as IsFixA: bool => 8 => get_is_fix_a,
          ),
        },
        utils as Utils => UtilsFunctions: {
          transfer_coin_to_sender as TransferCoinToSender => UtilsTransferCoinToSenderIndexes()
        },
      }
  },
}
