// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {FastTransferOrder} from "../FastTransferGateway.sol";

library OrderEncoder {
    function id(FastTransferOrder memory order) internal pure returns (bytes32) {
        return keccak256(encode(order));
    }

    function encode(FastTransferOrder memory order) internal pure returns (bytes memory) {
        return abi.encodePacked(
            order.sender,
            order.recipient,
            order.amountIn,
            order.amountOut,
            order.nonce,
            order.sourceDomain,
            order.destinationDomain,
            order.timeoutTimestamp,
            order.data
        );
    }

    function decode(bytes calldata orderBytes) internal pure returns (FastTransferOrder memory) {
        FastTransferOrder memory order;
        order.sender = bytes32(orderBytes[0:32]);
        order.recipient = bytes32(orderBytes[32:64]);
        order.amountIn = uint256(bytes32(orderBytes[64:96]));
        order.amountOut = uint256(bytes32(orderBytes[96:128]));
        order.nonce = uint32(bytes4(orderBytes[128:132]));
        order.sourceDomain = uint32(bytes4(orderBytes[132:136]));
        order.destinationDomain = uint32(bytes4(orderBytes[136:140]));
        order.timeoutTimestamp = uint64(bytes8(orderBytes[140:148]));
        order.data = bytes(orderBytes[148:]);

        return order;
    }
}
