// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.13;

import {Test, console} from "forge-std/Test.sol";
import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {IERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/IERC20Permit.sol";
import {ERC1967Proxy} from "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import {FastTransferGateway, FastTransferOrder, OrderFill} from "../src/FastTransferGateway.sol";
import {TypeCasts} from "../src/libraries/TypeCasts.sol";
import {OrderEncoder} from "../src/libraries/OrderEncoder.sol";
import {IPermit2} from "../src/interfaces/IPermit2.sol";
import {IMailbox} from "../src/interfaces/hyperlane/IMailbox.sol";
import {GoFastCaller} from "../src/GoFastMulticall.sol";

interface IUniswapV2Router02 {
    function swapExactTokensForTokens(
        uint256 amountIn,
        uint256 amountOutMin,
        address[] calldata path,
        address to,
        uint256 deadline
    ) external returns (uint256[] memory amounts);
}

contract FastTransferGatewayTest is Test {
    bytes32 constant TOKEN_PERMISSIONS_TYPEHASH = keccak256("TokenPermissions(address token,uint256 amount)");
    bytes32 constant PERMIT_TRANSFER_FROM_TYPEHASH = keccak256(
        "PermitTransferFrom(TokenPermissions permitted,address spender,uint256 nonce,uint256 deadline)TokenPermissions(address token,uint256 amount)"
    );

    uint256 arbitrumFork;

    IERC20 usdc = IERC20(0xaf88d065e77c8cC2239327C5EDb3A432268e5831);
    IERC20 weth = IERC20(0x82aF49447D8a07e3bd95BD0d56f35241523fBab1);

    IUniswapV2Router02 uniswapV2Router = IUniswapV2Router02(0x4752ba5DBc23f44D87826276BF6Fd6b1C372aD24);

    IPermit2 permit2 = IPermit2(0x000000000022D473030F116dDEE9F6B43aC78BA3);

    FastTransferGateway gateway;

    address user;
    address solver;
    address mailbox;
    address goFastCaller;

    function setUp() public {
        arbitrumFork = vm.createFork(vm.envString("RPC_URL"));

        vm.selectFork(arbitrumFork);
        vm.rollFork(242534997);

        user = address(1);
        solver = address(2);
        mailbox = address(0x979Ca5202784112f4738403dBec5D0F3B9daabB9);

        GoFastCaller _goFastCaller = new GoFastCaller();
        goFastCaller = address(_goFastCaller);
        FastTransferGateway gatewayImpl = new FastTransferGateway();
        ERC1967Proxy gatewayProxy = new ERC1967Proxy(
            address(gatewayImpl),
            abi.encodeWithSignature(
                "initialize(uint32,address,address,address,address,address,address)",
                1,
                address(this),
                address(usdc),
                mailbox,
                0x3d0BE14dFbB1Eb736303260c1724B6ea270c8Dc4,
                address(permit2),
                goFastCaller
            )
        );
        gateway = FastTransferGateway(address(gatewayProxy));
    }

    function test_submitAndSettle() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        // 1. User submits order to gateway
        bytes32 orderID = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        uint256 userBalanceAfter = usdc.balanceOf(user);
        assertEq(userBalanceAfter, 0);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, amountIn);

        // 2. Solver fills order on destination chain contract which sends a message back to the source chain
        // ...

        // 3. Mailbox receives message and calls gateway to release funds
        uint256 solverBalanceBefore = usdc.balanceOf(solver);

        bytes32[] memory orderIDs = new bytes32[](1);
        orderIDs[0] = orderID;

        _settleOrders(destinationDomain, destinationContract, orderIDs);

        uint256 solverBalanceAfter = usdc.balanceOf(solver);

        assertEq(solverBalanceAfter, solverBalanceBefore + amountIn);
    }

    function test_submitAndSettleMultipleOrders() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        bytes32 orderID1 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));
        bytes32 orderID2 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        uint256 userBalanceAfter = usdc.balanceOf(user);
        assertEq(userBalanceAfter, 0);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, amountIn * 2);

        uint256 solverBalanceBefore = usdc.balanceOf(solver);

        bytes32[] memory orderIDs = new bytes32[](2);
        orderIDs[0] = orderID1;
        orderIDs[1] = orderID2;

        _settleOrders(destinationDomain, destinationContract, orderIDs);

        uint256 solverBalanceAfter = usdc.balanceOf(solver);

        assertEq(solverBalanceAfter, solverBalanceBefore + (amountIn * 2));
    }

    function test_submitAndSettleMultipleOrdersSkipsAlreadySettledOrdersButDoesntRevert() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        bytes32 orderID1 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));
        bytes32 orderID2 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));
        bytes32 orderID3 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        uint256 userBalanceAfter = usdc.balanceOf(user);
        assertEq(userBalanceAfter, 0);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, amountIn * 3);

        uint256 solverBalanceBefore = usdc.balanceOf(solver);

        // settle order 1
        bytes32[] memory orderIDs = new bytes32[](1);
        orderIDs[0] = orderID1;

        _settleOrders(destinationDomain, destinationContract, orderIDs);

        uint256 solverBalanceAfter = usdc.balanceOf(solver);

        assertEq(solverBalanceAfter, solverBalanceBefore + amountIn);

        // settle all orders, this shouldn't fail even though order 1 is already settled
        orderIDs = new bytes32[](3);
        orderIDs[0] = orderID1;
        orderIDs[1] = orderID2;
        orderIDs[2] = orderID3;

        _settleOrders(destinationDomain, destinationContract, orderIDs);

        solverBalanceAfter = usdc.balanceOf(solver);

        assertEq(solverBalanceAfter, solverBalanceBefore + (amountIn * 3));
    }

    function test_revertSubmitOrderWhenRemoteDomainUnknown() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;

        deal(address(usdc), user, amountIn, true);

        // 1. User submits order to gateway
        vm.startPrank(user);

        usdc.approve(address(gateway), amountIn);

        vm.expectRevert("FastTransferGateway: destination domain not found");
        gateway.submitOrder(
            TypeCasts.addressToBytes32(user),
            TypeCasts.addressToBytes32(user),
            amountIn,
            amountOut,
            3,
            uint64(block.timestamp + 1 days),
            bytes("")
        );

        vm.stopPrank();
    }

    function test_revertSettlementWhenRemoteDomainUnknown() public {
        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, keccak256("orderID"));

        bytes memory _msg = abi.encodePacked(uint8(0), TypeCasts.addressToBytes32(solver), orderIDs);

        vm.prank(mailbox);
        vm.expectRevert("FastTransferGateway: origin domain not found");
        gateway.handle(1, keccak256("destinationChainContract"), _msg);
    }

    function test_revertSettlementWhenRemoteDomainNotCorrect() public {
        gateway.setRemoteDomain(1, keccak256("destinationChainContract"));

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, keccak256("orderID"));

        bytes memory _msg = abi.encodePacked(uint8(0), TypeCasts.addressToBytes32(solver), orderIDs);

        vm.prank(mailbox);
        vm.expectRevert("FastTransferGateway: invalid sender");
        gateway.handle(1, keccak256("invalidValue"), _msg);
    }

    function test_revertSettlementWhenCallerIsntMailbox() public {
        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, keccak256("orderID"));

        bytes memory _msg = abi.encodePacked(uint8(0), TypeCasts.addressToBytes32(solver), orderIDs);

        vm.expectRevert("FastTransferGateway: sender not mailbox");
        gateway.handle(1, keccak256("destinationChainContract"), _msg);
    }

    function test_skipsSettlementWhenOrderHasAlreadyBeenRefunded() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        // 1. User submits order to gateway
        bytes32 orderID = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, abi.encodePacked(uint8(1), orderID));

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, orderID);

        bytes memory _msg = abi.encodePacked(uint8(0), TypeCasts.addressToBytes32(solver), orderIDs);

        uint256 gatewayBalanceBefore = usdc.balanceOf(address(gateway));

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, _msg);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, gatewayBalanceBefore);
    }

    function test_submitAndRefund() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        // 1. User submits order to gateway
        bytes32 orderID = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        uint256 userBalanceAfter = usdc.balanceOf(user);
        assertEq(userBalanceAfter, 0);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, amountIn);

        bytes memory _msg = abi.encodePacked(uint8(1), orderID);

        uint256 userBalanceBeforeRefund = usdc.balanceOf(user);

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, _msg);

        uint256 userBalanceAfterRefund = usdc.balanceOf(user);
        assertEq(userBalanceAfterRefund, userBalanceBeforeRefund + amountIn);
    }

    function test_submitAndRefundMultipleOrders() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        // 1. User submits order to gateway
        bytes32 orderID1 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));
        bytes32 orderID2 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        uint256 userBalanceAfter = usdc.balanceOf(user);
        assertEq(userBalanceAfter, 0);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, amountIn * 2);

        bytes memory _msg = abi.encodePacked(uint8(1), orderID1, orderID2);

        uint256 userBalanceBeforeRefund = usdc.balanceOf(user);

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, _msg);

        uint256 userBalanceAfterRefund = usdc.balanceOf(user);
        assertEq(userBalanceAfterRefund, userBalanceBeforeRefund + (amountIn * 2));
    }

    function test_submitAndRefundSkipsAlreadySettledOrders() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        bytes32 orderID = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));
        bytes32 orderID2 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        bytes32[] memory orderIDs = new bytes32[](1);
        orderIDs[0] = orderID;

        _settleOrders(destinationDomain, destinationContract, orderIDs);

        uint256 userBalanceBeforeRefund = usdc.balanceOf(user);

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, abi.encodePacked(uint8(1), orderID, orderID2));

        uint256 userBalanceAfterRefund = usdc.balanceOf(user);
        assertEq(userBalanceAfterRefund, userBalanceBeforeRefund + amountIn);
    }

    function test_submitAndRefundSkipsAlreadyRefundedOrders() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 destinationDomain = 2;
        bytes32 destinationContract = TypeCasts.addressToBytes32(address(0xA));

        gateway.setRemoteDomain(destinationDomain, destinationContract);

        bytes32 orderID = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));
        bytes32 orderID2 = _submitOrder(amountIn, amountOut, destinationDomain, bytes(""));

        uint256 userBalanceBeforeRefund = usdc.balanceOf(user);

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, abi.encodePacked(uint8(1), orderID));

        vm.prank(mailbox);
        gateway.handle(destinationDomain, destinationContract, abi.encodePacked(uint8(1), orderID, orderID2));

        uint256 userBalanceAfterRefund = usdc.balanceOf(user);
        assertEq(userBalanceAfterRefund, userBalanceBeforeRefund + (amountIn * 2));
    }

    function test_submitWithPermit() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
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

        uint256 userBalanceBefore = usdc.balanceOf(alice);
        uint256 gatewayBalanceBefore = usdc.balanceOf(address(gateway));

        {
            vm.startPrank(alice);

            usdc.approve(address(permit2), type(uint256).max);

            bytes memory sig = _signPermit(permit, address(gateway), alicePk);

            gateway.submitOrderWithPermit(
                TypeCasts.addressToBytes32(alice),
                TypeCasts.addressToBytes32(alice),
                amountIn,
                amountOut,
                destinationDomain,
                uint64(block.timestamp + 1 days),
                uint64(block.timestamp + 1 days),
                bytes(""),
                sig
            );

            vm.stopPrank();
        }

        uint256 userBalanceAfter = usdc.balanceOf(alice);
        assertEq(userBalanceAfter, userBalanceBefore - amountIn);

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, gatewayBalanceBefore + amountIn);
    }

    function test_fillOrder() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, amountOut, true);

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        uint256 solverBalanceBefore = usdc.balanceOf(solver);
        uint256 gatewayBalanceBefore = usdc.balanceOf(address(gateway));
        uint256 recipientBalanceBefore = usdc.balanceOf(address(0xC));

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        gateway.fillOrder(solver, order);
        vm.stopPrank();

        uint256 solverBalanceAfter = usdc.balanceOf(solver);
        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        uint256 recipientBalanceAfter = usdc.balanceOf(address(0xC));

        assertEq(solverBalanceAfter, solverBalanceBefore - amountOut);
        assertEq(gatewayBalanceAfter, gatewayBalanceBefore);
        assertEq(recipientBalanceAfter, recipientBalanceBefore + amountOut);
    }

    function test_fillOrderWithData() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint256 expectedWethOut = 35987234755983635;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, amountOut, true);

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        address[] memory path = new address[](2);
        path[0] = address(usdc);
        path[1] = address(weth);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(uniswapV2Router)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: abi.encodeWithSelector(
                uniswapV2Router.swapExactTokensForTokens.selector,
                amountOut,
                expectedWethOut,
                path,
                TypeCasts.addressToBytes32(address(0xC)),
                block.timestamp + 1 days
            )
        });

        uint256 solverBalanceBefore = usdc.balanceOf(solver);
        uint256 gatewayBalanceBefore = usdc.balanceOf(address(gateway));
        uint256 recipientBalanceBefore = usdc.balanceOf(address(0xC));
        uint256 recipientWethBalanceBefore = weth.balanceOf(address(0xC));

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        gateway.fillOrder(solver, order);
        vm.stopPrank();

        uint256 solverBalanceAfter = usdc.balanceOf(solver);
        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        uint256 recipientBalanceAfter = usdc.balanceOf(address(0xC));
        uint256 recipientWethBalanceAfter = weth.balanceOf(address(0xC));

        assertEq(solverBalanceAfter, solverBalanceBefore - amountOut);
        assertEq(gatewayBalanceAfter, gatewayBalanceBefore);
        assertEq(recipientBalanceAfter, recipientBalanceBefore);
        assertEq(recipientWethBalanceAfter, recipientWethBalanceBefore + expectedWethOut);

        (, address orderFiller,) = gateway.orderFills(OrderEncoder.id(order));
        assertEq(orderFiller, solver);
    }

    function test_revertFillOrderCantTransferTokensFromAnotherUser() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, amountOut, true);
        deal(address(usdc), user, 5 ether, true);

        uint256 userBalanceBefore = usdc.balanceOf(user);

        bytes memory data =
            abi.encodeWithSelector(IERC20.transferFrom.selector, user, address(0x1337), uint256(5 ether));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(usdc)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: data
        });

        vm.prank(user);
        usdc.approve(address(gateway), 5 ether);

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        vm.expectRevert("ERC20: transfer amount exceeds allowance");
        gateway.fillOrder(solver, order);
        vm.stopPrank();

        assertEq(usdc.balanceOf(user), userBalanceBefore);
    }

    function test_revertFillOrderCantTransferTokensFromGateway() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, 5 ether);
        deal(address(usdc), address(gateway), 5 ether, true);

        bytes memory data = abi.encodeWithSelector(IERC20.transfer.selector, address(0x1337), uint256(5 ether));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(usdc)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: data
        });

        uint256 gatewayBalanceBefore = usdc.balanceOf(address(gateway));

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        vm.expectRevert("ERC20: transfer amount exceeds balance");
        gateway.fillOrder(solver, order);
        vm.stopPrank();

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, gatewayBalanceBefore);
    }

    function test_revertFillOrderCantCallMailbox() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, 5 ether);
        deal(address(usdc), address(gateway), 5 ether, true);

        bytes memory data = abi.encodeWithSelector(
            IMailbox.dispatch.selector, 1, TypeCasts.addressToBytes32(address(0x1337)), bytes("")
        );

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(mailbox)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: data
        });

        uint256 gatewayBalanceBefore = usdc.balanceOf(address(gateway));

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        vm.expectRevert("FastTransferGateway: order recipient cannot be mailbox");
        gateway.fillOrder(solver, order);
        vm.stopPrank();

        uint256 gatewayBalanceAfter = usdc.balanceOf(address(gateway));
        assertEq(gatewayBalanceAfter, gatewayBalanceBefore);
    }

    function test_revertFillOrderWhenOrderExpired() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, amountOut, true);

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp - 1 days),
            data: bytes("")
        });

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        vm.expectRevert("FastTransferGateway: order expired");
        gateway.fillOrder(solver, order);
        vm.stopPrank();
    }

    function test_revertFillOrderWhenOrderExpiredExact() public {
        uint256 amountIn = 100_000000;
        uint256 amountOut = 98_000000;
        uint32 sourceDomain = 1;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        deal(address(usdc), solver, amountOut, true);

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory order = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: amountIn,
            amountOut: amountOut,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp),
            data: bytes("")
        });

        vm.startPrank(solver);
        usdc.approve(address(gateway), amountOut);

        vm.expectRevert("FastTransferGateway: order expired");
        gateway.fillOrder(solver, order);
        vm.stopPrank();
    }

    function test_initiateSettlement() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        FastTransferOrder memory orderB = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 2,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut + orderB.amountOut, true);
        deal(solver, 1 ether);

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderA));
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderB));

        uint256 hyperlaneFee =
            gateway.quoteInitiateSettlement(sourceDomain, TypeCasts.addressToBytes32(solver), orderIDs);

        vm.startPrank(solver);

        usdc.approve(address(gateway), orderA.amountOut + orderB.amountOut);

        gateway.fillOrder(solver, orderA);
        gateway.fillOrder(solver, orderB);

        gateway.initiateSettlement{value: hyperlaneFee}(TypeCasts.addressToBytes32(solver), orderIDs);
        vm.stopPrank();
    }

    function test_revertInitiateSettlementUnknownOrder() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        FastTransferOrder memory orderB = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 2,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut + orderB.amountOut, true);
        deal(solver, 1 ether);

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, keccak256(abi.encode(orderA)));
        orderIDs = bytes.concat(orderIDs, keccak256(abi.encode(orderB)));

        uint256 hyperlaneFee =
            gateway.quoteInitiateSettlement(sourceDomain, TypeCasts.addressToBytes32(solver), orderIDs);

        vm.startPrank(solver);

        usdc.approve(address(gateway), orderA.amountOut + orderB.amountOut);

        gateway.fillOrder(solver, orderA);

        // don't fill orderB
        // gateway.fillOrder(orderB);

        vm.expectRevert("FastTransferGateway: order not filled");
        gateway.initiateSettlement{value: hyperlaneFee}(TypeCasts.addressToBytes32(solver), orderIDs);
        vm.stopPrank();
    }

    function test_revertInitiateSettlementWrongFiller() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        address solver2 = makeAddr("solver2");

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        FastTransferOrder memory orderB = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 2,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut, true);
        deal(solver, 1 ether);

        deal(address(usdc), solver2, orderB.amountOut, true);
        deal(solver2, 1 ether);

        // solver fills orderA
        vm.startPrank(solver);
        usdc.approve(address(gateway), orderA.amountOut);
        gateway.fillOrder(solver, orderA);
        vm.stopPrank();

        // solver2 fills orderB
        vm.startPrank(solver2);
        usdc.approve(address(gateway), orderB.amountOut);
        gateway.fillOrder(solver2, orderB);
        vm.stopPrank();

        vm.startPrank(solver);
        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderA));
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderB));

        uint256 hyperlaneFee =
            gateway.quoteInitiateSettlement(sourceDomain, TypeCasts.addressToBytes32(solver), orderIDs);

        usdc.approve(address(gateway), orderA.amountOut + orderB.amountOut);

        vm.expectRevert("FastTransferGateway: Unauthorized");
        gateway.initiateSettlement{value: hyperlaneFee}(TypeCasts.addressToBytes32(solver), orderIDs);
        vm.stopPrank();
    }

    function test_revertInitiateSettlementSourceDomainsDontMatch() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);
        gateway.setRemoteDomain(1, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        FastTransferOrder memory orderB = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 2,
            sourceDomain: 1,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut + orderB.amountOut, true);
        deal(solver, 1 ether);

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderA));
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderB));

        uint256 hyperlaneFee =
            gateway.quoteInitiateSettlement(sourceDomain, TypeCasts.addressToBytes32(solver), orderIDs);

        vm.startPrank(solver);

        usdc.approve(address(gateway), orderA.amountOut + orderB.amountOut);

        gateway.fillOrder(solver, orderA);
        gateway.fillOrder(solver, orderB);

        vm.expectRevert("FastTransferGateway: Source domains must match");
        gateway.initiateSettlement{value: hyperlaneFee}(TypeCasts.addressToBytes32(solver), orderIDs);
        vm.stopPrank();
    }

    function test_revertInitiateSettlementOnDuplicateOrder() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 days),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut + orderA.amountOut, true);
        deal(solver, 1 ether);

        bytes memory orderIDs;
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderA));
        orderIDs = bytes.concat(orderIDs, OrderEncoder.id(orderA));

        uint256 hyperlaneFee =
            gateway.quoteInitiateSettlement(sourceDomain, TypeCasts.addressToBytes32(solver), orderIDs);

        vm.startPrank(solver);

        usdc.approve(address(gateway), orderA.amountOut + orderA.amountOut);

        gateway.fillOrder(solver, orderA);

        vm.expectRevert("FastTransferGateway: duplicate order");
        gateway.initiateSettlement{value: hyperlaneFee}(TypeCasts.addressToBytes32(solver), orderIDs);
        vm.stopPrank();
    }

    function test_initiateTimeout() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp - 1 hours),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut, true);
        deal(solver, 1 ether);

        FastTransferOrder[] memory orders = new FastTransferOrder[](1);
        orders[0] = orderA;

        uint256 hyperlaneFee = gateway.quoteInitiateTimeout(sourceDomain, orders);

        vm.startPrank(solver);
        gateway.initiateTimeout{value: hyperlaneFee}(orders);
        vm.stopPrank();
    }

    function test_revertInitiateTimeoutOrderNotExpired() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 hours),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut, true);
        deal(solver, 1 ether);

        FastTransferOrder[] memory orders = new FastTransferOrder[](1);
        orders[0] = orderA;

        uint256 hyperlaneFee = gateway.quoteInitiateTimeout(sourceDomain, orders);

        vm.startPrank(solver);
        vm.expectRevert("FastTransferGateway: order not timed out");
        gateway.initiateTimeout{value: hyperlaneFee}(orders);
        vm.stopPrank();
    }

    function test_revertInitiateTimeoutOrderFilled() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp + 1 hours),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut, true);
        deal(solver, 1 ether);

        FastTransferOrder[] memory orders = new FastTransferOrder[](1);
        orders[0] = orderA;

        vm.startPrank(solver);
        usdc.approve(address(gateway), orderA.amountOut);
        gateway.fillOrder(solver, orderA);
        vm.stopPrank();

        vm.warp(block.timestamp + 2 hours);

        vm.startPrank(solver);
        uint256 hyperlaneFee = gateway.quoteInitiateTimeout(sourceDomain, orders);
        vm.expectRevert("FastTransferGateway: order filled");
        gateway.initiateTimeout{value: hyperlaneFee}(orders);
        vm.stopPrank();
    }

    function test_revertInitiateTimeoutSourceDomainsDontMatch() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);
        gateway.setRemoteDomain(1, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp - 1 hours),
            data: bytes("")
        });

        FastTransferOrder memory orderB = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 2,
            sourceDomain: 1,
            destinationDomain: 1,
            timeoutTimestamp: uint64(block.timestamp - 1 hours),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut, true);
        deal(solver, 1 ether);

        FastTransferOrder[] memory orders = new FastTransferOrder[](2);
        orders[0] = orderA;
        orders[1] = orderB;

        uint256 hyperlaneFee = gateway.quoteInitiateTimeout(sourceDomain, orders);

        vm.startPrank(solver);
        vm.expectRevert("FastTransferGateway: Source domains must match");
        gateway.initiateTimeout{value: hyperlaneFee}(orders);
        vm.stopPrank();
    }

    function test_initiateTimeoutRevertsIfDestinationDomainIsNotTheLocalDomain() public {
        uint32 sourceDomain = 8453;
        bytes32 sourceContract = TypeCasts.addressToBytes32(address(0xB));

        gateway.setRemoteDomain(sourceDomain, sourceContract);

        FastTransferOrder memory orderA = FastTransferOrder({
            sender: TypeCasts.addressToBytes32(address(0xB)),
            recipient: TypeCasts.addressToBytes32(address(0xC)),
            amountIn: 100_000000,
            amountOut: 98_000000,
            nonce: 1,
            sourceDomain: sourceDomain,
            destinationDomain: 3,
            timeoutTimestamp: uint64(block.timestamp - 1 hours),
            data: bytes("")
        });

        deal(address(usdc), solver, orderA.amountOut, true);
        deal(solver, 1 ether);

        FastTransferOrder[] memory orders = new FastTransferOrder[](1);
        orders[0] = orderA;

        uint256 hyperlaneFee = gateway.quoteInitiateTimeout(sourceDomain, orders);

        vm.startPrank(solver);
        vm.expectRevert("FastTransferGateway: invalid local domain");
        gateway.initiateTimeout{value: hyperlaneFee}(orders);
        vm.stopPrank();
    }

    function _submitOrder(uint256 amountIn, uint256 amountOut, uint32 destinationDomain, bytes memory data)
        internal
        returns (bytes32)
    {
        deal(address(usdc), user, amountIn, true);

        vm.startPrank(user);

        usdc.approve(address(gateway), amountIn);

        bytes32 orderID = gateway.submitOrder(
            TypeCasts.addressToBytes32(user),
            TypeCasts.addressToBytes32(user),
            amountIn,
            amountOut,
            destinationDomain,
            uint64(block.timestamp + 1 days),
            data
        );

        vm.stopPrank();

        return orderID;
    }

    function _settleOrders(uint32 _destinationDomain, bytes32 _destinationContract, bytes32[] memory _orderIDs)
        internal
    {
        bytes memory orderIDs;
        for (uint256 i = 0; i < _orderIDs.length; i++) {
            orderIDs = bytes.concat(orderIDs, _orderIDs[i]);
        }

        bytes memory _msg = abi.encodePacked(uint8(0), TypeCasts.addressToBytes32(solver), orderIDs);

        vm.prank(mailbox);
        gateway.handle(_destinationDomain, _destinationContract, _msg);
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
