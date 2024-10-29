// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Script, console} from "forge-std/Script.sol";

import {FastTransferGateway, FastTransferOrder} from "../../src/FastTransferGateway.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {TypeCasts} from "../../src/libraries/TypeCasts.sol";
import {GoFastERC7683, OrderData} from "../../src/GoFastERC7683.sol";
import {OnchainCrossChainOrder, GaslessCrossChainOrder} from "../../src/erc7683/ERC7683.sol";

contract DeployScript is Script {
    function setUp() public {}

    function run() public {
        vm.startBroadcast();

        GoFastERC7683 goFast = GoFastERC7683(0x92188c8200869b7bfB9A867C545ea723bD8AfEA1);

        address inputToken = 0xaf88d065e77c8cC2239327C5EDb3A432268e5831; // USDC
        bytes memory outputToken = bytes("ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4");
        address recipient = address(0x4A92560C84e522819d9785A13d6adC44CDDeFaC7);
        uint256 amount = 500000;
        uint32 localDomain = 42161;
        uint32 destinationDomain = 875;
        bytes memory data = bytes("");

        uint256 balance = IERC20(inputToken).balanceOf(msg.sender);
        require(balance >= amount, "Insufficient balance");

        OnchainCrossChainOrder memory order = OnchainCrossChainOrder({
            fillDeadline: 0,
            orderDataType: goFast.GO_FAST_ORDER_DATA_TYPE_HASH(),
            orderData: abi.encode(
                OrderData(
                    TypeCasts.addressToBytes32(msg.sender), // sender
                    TypeCasts.addressToBytes32(recipient), // recipient
                    abi.encode(inputToken), // inputToken
                    outputToken,
                    amount, // amountIn
                    amount, // amountOut
                    localDomain, // sourceDomain
                    destinationDomain, // destinationDomain
                    uint64(block.timestamp + 1 days), // timeoutTimestamp
                    data
                )
            )
        });

        uint256 allowance = IERC20(inputToken).allowance(msg.sender, address(goFast));
        if (allowance < amount) {
            IERC20(inputToken).approve(address(goFast), type(uint256).max);
        }

        goFast.open(order);

        vm.stopBroadcast();
    }
}
