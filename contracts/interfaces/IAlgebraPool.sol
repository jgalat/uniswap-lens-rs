// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity ^0.8.0;

interface IAlgebraPool {
    function ticks(
        int24 tick
    )
        external
        view
        returns (
            uint256 liquidityTotal,
            int128 liquidityDelta,
            int24 prevTick,
            int24 nextTick,
            uint256 outerFeeGrowth0Token,
            uint256 outerFeeGrowth1Token
        );

    function tickTable(int16 wordPosition) external view returns (uint256);
}
