// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {GoFastERC7683} from "../../src/GoFastERC7683.sol";

contract UpgradeScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        GoFastERC7683 newContractImpl = new GoFastERC7683();

        GoFastERC7683 proxyContract = GoFastERC7683(0x92188c8200869b7bfB9A867C545ea723bD8AfEA1);

        proxyContract.upgradeToAndCall(address(newContractImpl), bytes(""));

        vm.stopBroadcast();
    }
}
