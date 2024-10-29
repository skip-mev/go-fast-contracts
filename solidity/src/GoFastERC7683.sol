// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {console2} from "forge-std/console2.sol";

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Permit.sol";
import {OwnableUpgradeable} from "@openzeppelin/contracts-upgradeable/access/OwnableUpgradeable.sol";
import {Initializable} from "@openzeppelin/contracts-upgradeable/proxy/utils/Initializable.sol";
import {UUPSUpgradeable} from "@openzeppelin/contracts/proxy/utils/UUPSUpgradeable.sol";
import {ReentrancyGuardUpgradeable} from "@openzeppelin/contracts-upgradeable/utils/ReentrancyGuardUpgradeable.sol";

import {TypeCasts} from "./libraries/TypeCasts.sol";
import {OrderEncoder} from "./libraries/OrderEncoder.sol";
import {IPermit2} from "./interfaces/IPermit2.sol";
import {FastTransferOrder, FastTransferGateway} from "./FastTransferGateway.sol";

import {
    OnchainCrossChainOrder,
    ResolvedCrossChainOrder,
    Output,
    FillInstruction,
    GaslessCrossChainOrder
} from "./erc7683/ERC7683.sol";

struct OrderData {
    bytes32 sender;
    bytes32 recipient;
    bytes inputToken;
    bytes outputToken;
    uint256 amountIn;
    uint256 amountOut;
    uint32 sourceDomain;
    uint32 destinationDomain;
    uint64 timeoutTimestamp;
    bytes data;
}

contract GoFastERC7683 is Initializable, UUPSUpgradeable, OwnableUpgradeable, ReentrancyGuardUpgradeable {
    address public gateway;
    IPermit2 public PERMIT2;

    mapping(uint32 => bytes32) public destinationSettlers;

    bytes constant GO_FAST_ORDER_DATA_TYPE = abi.encodePacked(
        "OrderData(",
        "bytes32 sender,",
        "bytes32 recipient,",
        "bytes32 inputToken,",
        "bytes32 outputToken,",
        "uint256 amountIn,",
        "uint256 amountOut,",
        "uint32 sourceDomain,",
        "uint32 destinationDomain,",
        "uint64 timeoutTimestamp,",
        "bytes data)"
    );

    bytes32 public constant GO_FAST_ORDER_DATA_TYPE_HASH = keccak256(GO_FAST_ORDER_DATA_TYPE);

    event Open(bytes32 indexed orderID, ResolvedCrossChainOrder resolvedOrder);

    constructor() {
        _disableInitializers();
    }

    function initialize(address _owner, address _gateway, address _permit2) external initializer {
        __Ownable_init(_owner);
        gateway = _gateway;
        PERMIT2 = IPermit2(_permit2);
    }

    function addDestinationSettler(uint32 destinationDomain, bytes32 destinationSettler) external onlyOwner {
        destinationSettlers[destinationDomain] = destinationSettler;
    }

    function open(OnchainCrossChainOrder calldata order) external nonReentrant {
        (ResolvedCrossChainOrder memory resolvedOrder, OrderData memory orderData) = _resolve(order);

        if (orderData.inputToken.length != 32) {
            revert("invalid input token");
        }

        address inputToken = TypeCasts.bytes32ToAddress(bytes32(orderData.inputToken));

        IERC20(inputToken).transferFrom(msg.sender, address(this), orderData.amountIn);

        IERC20(inputToken).approve(gateway, orderData.amountIn);

        bytes32 orderID = FastTransferGateway(gateway).submitOrder(
            orderData.sender,
            orderData.recipient,
            orderData.amountIn,
            orderData.amountOut,
            orderData.destinationDomain,
            orderData.timeoutTimestamp,
            orderData.data
        );

        emit Open(orderID, resolvedOrder);
    }

    function openFor(GaslessCrossChainOrder calldata order, bytes calldata signature, bytes calldata) external {
        (ResolvedCrossChainOrder memory resolvedOrder, OrderData memory orderData) = _resolveFor(order);

        if (orderData.inputToken.length != 32) {
            revert("invalid input token");
        }

        address inputToken = TypeCasts.bytes32ToAddress(bytes32(orderData.inputToken));

        _permitTransferFrom(inputToken, orderData.amountIn, order.openDeadline, uint32(order.nonce), signature);

        IERC20(inputToken).approve(gateway, orderData.amountIn);

        bytes32 orderID = FastTransferGateway(gateway).submitOrder(
            orderData.sender,
            orderData.recipient,
            orderData.amountIn,
            orderData.amountOut,
            orderData.destinationDomain,
            orderData.timeoutTimestamp,
            orderData.data
        );

        emit Open(orderID, resolvedOrder);
    }

    function fill(bytes32, bytes calldata originData, bytes calldata) external {
        FastTransferOrder memory order = OrderEncoder.decode(originData);

        address inputToken = FastTransferGateway(gateway).token();

        IERC20(inputToken).transferFrom(msg.sender, address(this), order.amountOut);

        IERC20(inputToken).approve(gateway, order.amountOut);

        FastTransferGateway(gateway).fillOrder(msg.sender, order);
    }

    function _resolve(OnchainCrossChainOrder calldata order)
        internal
        view
        returns (ResolvedCrossChainOrder memory resolvedOrder, OrderData memory orderData)
    {
        if (order.orderDataType != GO_FAST_ORDER_DATA_TYPE_HASH) {
            revert("invalid order data type");
        }

        orderData = abi.decode(order.orderData, (OrderData));

        bytes32 destinationSettler = destinationSettlers[orderData.destinationDomain];
        require(destinationSettler != bytes32(0), "invalid destination domain");

        Output[] memory maxSpent = new Output[](1);
        maxSpent[0] = Output({
            token: orderData.outputToken,
            amount: orderData.amountOut,
            recipient: orderData.recipient,
            chainId: orderData.destinationDomain
        });

        Output[] memory minReceived = new Output[](1);
        minReceived[0] = Output({
            token: orderData.inputToken,
            amount: orderData.amountIn,
            recipient: bytes32(0),
            chainId: orderData.sourceDomain
        });

        FillInstruction[] memory fillInstructions = new FillInstruction[](1);
        fillInstructions[0] = FillInstruction({
            destinationChainId: orderData.destinationDomain,
            destinationSettler: destinationSettler,
            originData: OrderEncoder.encode(
                FastTransferOrder({
                    sender: orderData.sender,
                    recipient: orderData.recipient,
                    amountIn: orderData.amountIn,
                    amountOut: orderData.amountOut,
                    nonce: FastTransferGateway(gateway).nonce(),
                    sourceDomain: orderData.sourceDomain,
                    destinationDomain: orderData.destinationDomain,
                    timeoutTimestamp: orderData.timeoutTimestamp,
                    data: orderData.data
                })
            )
        });

        resolvedOrder = ResolvedCrossChainOrder({
            user: msg.sender,
            originChainId: orderData.sourceDomain,
            openDeadline: type(uint32).max, // no deadline since the user is sending it
            fillDeadline: order.fillDeadline,
            minReceived: minReceived,
            maxSpent: maxSpent,
            fillInstructions: fillInstructions
        });
    }

    function _resolveFor(GaslessCrossChainOrder calldata order)
        internal
        view
        returns (ResolvedCrossChainOrder memory resolvedOrder, OrderData memory orderData)
    {
        if (order.originSettler != gateway) {
            revert("invalid origin settler");
        }

        if (order.orderDataType != GO_FAST_ORDER_DATA_TYPE_HASH) {
            revert("invalid order data type");
        }

        orderData = abi.decode(order.orderData, (OrderData));

        Output[] memory maxSpent = new Output[](1);
        maxSpent[0] = Output({
            // TODO: i think this is supposed to be the token on the destination domain
            token: orderData.outputToken,
            amount: orderData.amountOut,
            recipient: orderData.recipient,
            chainId: orderData.destinationDomain
        });

        Output[] memory minReceived = new Output[](1);
        minReceived[0] = Output({
            token: orderData.inputToken,
            amount: orderData.amountIn,
            recipient: bytes32(0),
            chainId: orderData.sourceDomain
        });

        FillInstruction[] memory fillInstructions = new FillInstruction[](1);

        resolvedOrder = ResolvedCrossChainOrder({
            user: msg.sender,
            originChainId: orderData.sourceDomain,
            openDeadline: type(uint32).max, // no deadline since the user is sending it
            fillDeadline: order.fillDeadline,
            minReceived: minReceived,
            maxSpent: maxSpent,
            fillInstructions: fillInstructions
        });
    }

    function _permitTransferFrom(
        address token,
        uint256 amount,
        uint256 deadline,
        uint32 nonce,
        bytes calldata signature
    ) internal {
        PERMIT2.permitTransferFrom(
            IPermit2.PermitTransferFrom({
                permitted: IPermit2.TokenPermissions({token: IERC20(token), amount: amount}),
                nonce: nonce,
                deadline: deadline
            }),
            IPermit2.SignatureTransferDetails({to: address(this), requestedAmount: amount}),
            msg.sender,
            signature
        );
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}
}
