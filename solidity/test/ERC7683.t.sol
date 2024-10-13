// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console, Vm} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Permit.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import {FastTransferGateway, FastTransferOrder, OrderFill} from "../src/FastTransferGateway.sol";
import {TypeCasts} from "../src/libraries/TypeCasts.sol";
import {IPermit2} from "../src/interfaces/IPermit2.sol";
import {OnchainCrossChainOrder, GaslessCrossChainOrder} from "../src/erc7683/ERC7683.sol";
import {GoFastERC7683, OrderData} from "../src/GoFastERC7683.sol";
import {OrderEncoder} from "../src/libraries/OrderEncoder.sol";

interface IUniswapV2Router02 {
    function swapExactTokensForTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external returns (uint256[] memory amounts);
}

contract ERC7683Test is Test {
    using OrderEncoder for bytes;

    bytes32 constant TOKEN_PERMISSIONS_TYPEHASH = keccak256("TokenPermissions(address token,uint256 amount)");
    bytes32 constant PERMIT_TRANSFER_FROM_TYPEHASH = keccak256(
        "PermitTransferFrom(TokenPermissions permitted,address spender,uint256 nonce,uint256 deadline)TokenPermissions(address token,uint256 amount)"
    );

    uint256 arbitrumFork;

    IERC20 usdc = IERC20(0xaf88d065e77c8cC2239327C5EDb3A432268e5831);

    IPermit2 permit2 = IPermit2(0x000000000022D473030F116dDEE9F6B43aC78BA3);

    FastTransferGateway gateway;
    GoFastERC7683 goFastERC7683;
    address user;
    address solver;
    address mailbox;

    bytes32 OPEN_EVENT_TOPIC = bytes32(0x3f37fc7497ab35c3d02f3a854d5ca73476fcc1848d674e23af4aa8414d276005);

    function setUp() public {
        arbitrumFork = vm.createFork(vm.envString("RPC_URL"));

        vm.selectFork(arbitrumFork);
        vm.rollFork(242534997);

        user = address(1);
        solver = address(2);
        mailbox = address(0x979Ca5202784112f4738403dBec5D0F3B9daabB9);

        FastTransferGateway gatewayImpl = new FastTransferGateway();
        ERC1967Proxy gatewayProxy = new ERC1967Proxy(
            address(gatewayImpl),
            abi.encodeWithSignature(
                "initialize(uint32,address,address,address,address)",
                1,
                address(this),
                address(usdc),
                mailbox,
                address(permit2)
            )
        );
        gateway = FastTransferGateway(address(gatewayProxy));

        GoFastERC7683 goFastERC7683Impl = new GoFastERC7683();
        ERC1967Proxy goFastERC7683Proxy = new ERC1967Proxy(
            address(goFastERC7683Impl),
            abi.encodeWithSignature(
                "initialize(address,address,address)", address(this), address(gateway), address(permit2)
            )
        );
        goFastERC7683 = GoFastERC7683(address(goFastERC7683Proxy));
    }

    function test_open() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 90_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        goFastERC7683.addDestinationSettler(destinationDomain, keccak256("destination_7683_contract"));

        OnchainCrossChainOrder memory order = OnchainCrossChainOrder({
            fillDeadline: 0,
            orderDataType: goFastERC7683.GO_FAST_ORDER_DATA_TYPE_HASH(),
            orderData: abi.encode(
                OrderData(
                    TypeCasts.addressToBytes32(user), // sender
                    TypeCasts.addressToBytes32(user), // recipient
                    abi.encode(TypeCasts.addressToBytes32(address(usdc))), // inputToken
                    abi.encode(TypeCasts.addressToBytes32(address(usdc))), // outputToken
                    amountIn, // amountIn
                    amountOut, // amountOut
                    1, // sourceDomain
                    2, // destinationDomain
                    block.timestamp + 1 days, // timeoutTimestamp
                    bytes("") // data
                )
            )
        });

        deal(address(usdc), user, amountIn, true);

        vm.startPrank(user);

        usdc.approve(address(goFastERC7683), amountIn);

        vm.recordLogs();

        goFastERC7683.open(order);

        bytes32 orderID = _getOrderIDFromLogs();

        (bytes32 _sender, uint256 _nonce, uint32 _destinationDomain, uint256 _amount) =
            gateway.settlementDetails(orderID);

        assertEq(_sender, TypeCasts.addressToBytes32(user));
        assertEq(_nonce, 1);
        assertEq(_destinationDomain, 2);
        assertEq(_amount, amountIn);
    }

    function test_openFor() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 90_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        (address alice, uint256 alicePk) = makeAddrAndKey("alice");

        deal(address(usdc), alice, amountIn, true);

        IPermit2.PermitTransferFrom memory permit = IPermit2.PermitTransferFrom({
            permitted: IPermit2.TokenPermissions({token: IERC20(address(usdc)), amount: amountIn}),
            nonce: 1,
            deadline: block.timestamp + 1 days
        });

        GaslessCrossChainOrder memory order = GaslessCrossChainOrder({
            originSettler: address(gateway),
            user: user,
            nonce: 1,
            originChainId: 1,
            openDeadline: uint32(block.timestamp + 1 days),
            fillDeadline: uint32(block.timestamp + 2 days),
            orderDataType: goFastERC7683.GO_FAST_ORDER_DATA_TYPE_HASH(),
            orderData: abi.encode(
                OrderData(
                    TypeCasts.addressToBytes32(user), // sender
                    TypeCasts.addressToBytes32(user), // recipient
                    abi.encode(TypeCasts.addressToBytes32(address(usdc))), // inputToken
                    abi.encode(TypeCasts.addressToBytes32(address(usdc))), // outputToken
                    amountIn, // amountIn
                    amountOut, // amountOut
                    1, // sourceDomain
                    2, // destinationDomain
                    block.timestamp + 1 days, // timeoutTimestamp
                    bytes("") // data
                )
            )
        });

        vm.startPrank(alice);

        usdc.approve(address(permit2), type(uint256).max);

        bytes memory sig = _signPermit(permit, address(goFastERC7683), alicePk);

        vm.recordLogs();

        goFastERC7683.openFor(order, sig, bytes(""));

        bytes32 orderID = _getOrderIDFromLogs();

        (bytes32 _sender, uint256 _nonce, uint32 _destinationDomain, uint256 _amount) =
            gateway.settlementDetails(orderID);

        assertEq(_sender, TypeCasts.addressToBytes32(user));
        assertEq(_nonce, 1);
        assertEq(_destinationDomain, 2);
        assertEq(_amount, amountIn);
    }

    function test_fill() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 90_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: block.timestamp + 1 days,
            data: bytes("")
        });

        deal(address(usdc), solver, amountOut, true);

        bytes32 orderID = OrderEncoder.id(order);

        vm.startPrank(solver);
        usdc.approve(address(goFastERC7683), amountOut);
        goFastERC7683.fill(orderID, OrderEncoder.encode(order), bytes(""));

        (bytes32 _orderID, address _filler, uint32 _sourceDomain) = gateway.orderFills(orderID);

        assertEq(_orderID, orderID);
        assertEq(_filler, solver);
        assertEq(_sourceDomain, sourceDomain);
    }

    function _getOrderIDFromLogs() internal returns (bytes32) {
        Vm.Log[] memory logs = vm.getRecordedLogs();

        bytes32 orderID;

        for (uint256 i = 0; i < logs.length; i++) {
            Vm.Log memory log = logs[i];

            // // Open(bytes32 indexed orderId, ResolvedCrossChainOrder resolvedOrder)

            if (log.topics[0] != OPEN_EVENT_TOPIC) {
                continue;
            }

            orderID = log.topics[1];
        }

        return orderID;
    }

    function _signPermit(IPermit2.PermitTransferFrom memory permit, address spender, uint256 signerKey)
        internal
        view
        returns (bytes memory sig)
    {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(signerKey, _getEIP712Hash(permit, spender));
        return abi.encodePacked(r, s, v);
    }

    function _getEIP712Hash(IPermit2.PermitTransferFrom memory permit, address spender)
        internal
        view
        returns (bytes32 h)
    {
        return keccak256(
            abi.encodePacked(
                "\x19\x01",
                permit2.DOMAIN_SEPARATOR(),
                keccak256(
                    abi.encode(
                        PERMIT_TRANSFER_FROM_TYPEHASH,
                        keccak256(
                            abi.encode(TOKEN_PERMISSIONS_TYPEHASH, permit.permitted.token, permit.permitted.amount)
                        ),
                        spender,
                        permit.nonce,
                        permit.deadline
                    )
                )
            )
        );
    }
}
