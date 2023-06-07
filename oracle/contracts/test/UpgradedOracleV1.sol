// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.18;

import "../OracleV1.sol";

contract UpgradedOracleV1 is OracleV1 {
    function version() public pure override virtual returns (uint256) {
        return 2; 
    }
}
