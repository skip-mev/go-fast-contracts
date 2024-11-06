// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {FastTransferOrder} from "../src/FastTransferGateway.sol";
import {OrderEncoder} from "../src/libraries/OrderEncoder.sol";

contract PrintOrderID is Script {
    function run() public pure {
        FastTransferOrder memory order = FastTransferOrder({
            sender: keccak256("order_sender"),
            recipient: keccak256("order_recipient"),
            amountIn: 1_000000,
            amountOut: 2_000000,
            nonce: 5,
            sourceDomain: 1,
            destinationDomain: 2,
            timeoutTimestamp: 1234567890,
            data: bytes("order_data")
        });

        console.logBytes32(OrderEncoder.id(order));
    }
}
