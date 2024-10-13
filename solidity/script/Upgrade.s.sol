// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {FastTransferGateway, FastTransferOrder} from "../src/FastTransferGateway.sol";
import {TypeCasts} from "../src/libraries/TypeCasts.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract UpgradeScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        FastTransferGateway newGatewayImpl = new FastTransferGateway();

        FastTransferGateway gateway = FastTransferGateway(0xD594ff25a22416C0E9D2F8fD0a7166380f338977);

        gateway.upgradeToAndCall(address(newGatewayImpl), bytes(""));

        vm.stopBroadcast();
    }
}
