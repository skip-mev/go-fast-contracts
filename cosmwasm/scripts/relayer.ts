import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import {
  createPublicClient,
  http,
  Log,
  decodeEventLog,
  decodeAbiParameters,
} from "viem";
import { arbitrum } from "viem/chains";
import { FAST_TRANSFER_GATEWAY_ABI, MAILBOX_ABI } from "./abi";

let client: SigningCosmWasmClient;
const CHAIN_PREFIX = "neutron";
const RPC_URL = "https://neutron-rpc.polkachu.com";
const tokenDenom =
  "ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81";

async function onOrderSubmitted(logs: Log[]) {
  for (const log of logs) {
    try {
      const result = decodeEventLog({
        abi: FAST_TRANSFER_GATEWAY_ABI,
        data: log.data,
        topics: log.topics,
      });

      if (result.eventName !== "OrderSubmitted") {
        continue;
      }

      console.log("new order detected");

      const values = decodeAbiParameters(
        [
          { name: "sender", type: "bytes32" },
          { name: "recipient", type: "bytes32" },
          { name: "amount", type: "uint256" },
          { name: "nonce", type: "uint256" },
        ],
        result.args.order
      );

      const fillMessage = {
        fill_order: {
          order: {
            sender: values[0].replace("0x", ""),
            recipient: values[1].replace("0x", ""),
            amount: values[2].toString(),
            nonce: Number(values[3]),
          },
        },
      };

      const gasFee = { denom: "untrn", amount: "5180" };

      console.log("submitting fill transaction...");

      const fillTx = await client.execute(
        "neutron1f4h9nn3hv0q7fr7sze4zfkagl8vr8h03v2u3vy",
        "neutron10m9k9appv5lh6t465m6mc0t3qxhw2ma4egfz4h9xqsevnqcqmwts6ujh4r",
        fillMessage,
        {
          amount: [gasFee],
          gas: "800000",
        },
        undefined,
        [{ denom: tokenDenom, amount: values[2].toString() }]
      );

      console.log("filled", fillTx.transactionHash);
    } catch (err) {
      // console.error(err);
    }
  }
}

async function main() {
  const EVM_RPC_URL = process.env.EVM_RPC_URL;
  if (!EVM_RPC_URL) {
    throw new Error("EVM_RPC_URL is not set");
  }

  const DEPLOYER_MNEMONIC = process.env.DEPLOYER_MNEMONIC;
  if (!DEPLOYER_MNEMONIC) {
    throw new Error("DEPLOYER_MNEMONIC is not set");
  }

  const signer = await DirectSecp256k1HdWallet.fromMnemonic(DEPLOYER_MNEMONIC, {
    prefix: CHAIN_PREFIX,
  });

  const accounts = await signer.getAccounts();
  const signerAddress = accounts[0].address;

  console.log(signerAddress);

  client = await SigningCosmWasmClient.connectWithSigner(RPC_URL, signer);

  const publicClient = createPublicClient({
    chain: arbitrum,
    transport: http(EVM_RPC_URL),
  });

  // console.log("relayer started...");

  // const unwatch = publicClient.watchContractEvent({
  //   address: "0xf7b86fDee755f1821A6A7467ebf75A2BF7Aea227",
  //   abi: FAST_TRANSFER_GATEWAY_ABI,
  //   onLogs: onOrderSubmitted,
  // });

  const height = await publicClient.getBlockNumber();

  const result = await publicClient.getContractEvents({
    address: "0x979Ca5202784112f4738403dBec5D0F3B9daabB9",
    abi: MAILBOX_ABI,
    toBlock: height,
    fromBlock: height - BigInt(10000000),
    eventName: "ProcessId",
    args: {
      messageId:
        "0x4ff1e1ab8736943aa8cf2221b65c29f87f40ee72c16b508d83a235b5ff30d9f9",
    },
    // fromBlock
  });

  console.log(result.length);

  // const txReceipt = await publicClient.getTransactionReceipt({
  //   hash: "0xf46a6b4edc92b27513b789adf1a3891c7dd251db92902132abfa0869f4e612d1",
  // });

  // onOrderSubmitted(txReceipt.logs);
}

main().catch(console.error);
