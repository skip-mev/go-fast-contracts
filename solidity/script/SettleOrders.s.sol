// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {FastTransferGateway, FastTransferOrder} from "../src/FastTransferGateway.sol";
import {TypeCasts} from "../src/libraries/TypeCasts.sol";
import {IMailbox} from "../src/interfaces/hyperlane/IMailbox.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract SettleOrdersScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

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

        bytes32 repaymentAddress = TypeCasts.addressToBytes32(msg.sender);

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, keccak256(abi.encode(order)));

        bytes memory hyperlaneMessage = abi.encodePacked(uint8(0), repaymentAddress, orderIDs);

        uint256 hyperlaneFee = IMailbox(gateway.mailbox()).quoteDispatch(
            order.sourceDomain, TypeCasts.addressToBytes32(0x83eFe03da48cF12a258c5bb210097E8b0aB2F61F), hyperlaneMessage
        );

        gateway.initiateSettlement{value: hyperlaneFee}(repaymentAddress, orderIDs);

        vm.stopBroadcast();
    }
}
