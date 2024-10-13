// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {FastTransferGateway, FastTransferOrder} from "../src/FastTransferGateway.sol";
import {TypeCasts} from "../src/libraries/TypeCasts.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract FillOrderScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        address token = address(0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913); // USDC

        FastTransferGateway gateway = FastTransferGateway(0x80a428AEd33FeC3867850c75Ad8b6bB0Ec1270cA);

        FastTransferOrder memory order = FastTransferOrder({
            sender: hex"00000000000000000000000024a9267ce9e0a8f4467b584fdda12baf1df772b5",
            recipient: hex"00000000000000000000000024a9267ce9e0a8f4467b584fdda12baf1df772b5",
            amountIn: 5_000000,
            amountOut: 5_000000,
            nonce: 1,
            sourceDomain: 42161,
            destinationDomain: 8453,
            timeoutTimestamp: 1728125702,
            data: bytes("")
        });

        IERC20(token).approve(address(gateway), order.amountOut);

        gateway.fillOrder(msg.sender, order);

        vm.stopBroadcast();
    }
}
