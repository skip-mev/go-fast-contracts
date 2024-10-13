import "dotenv/config";
import {
  createPublicClient,
  http,
  decodeAbiParameters,
  parseAbiParameters,
} from "viem";
import { arbitrum } from "viem/chains";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { toBech32, fromHex, toBase64 } from "@cosmjs/encoding";
import { GasPrice, calculateFee, coin } from "@cosmjs/stargate";
import { Coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { GOFAST_7683_ABI } from "../abi";
import { ResolvedCrossChainOrder, FastTransferOrder } from "./types";

const ERC7683_ADDRESS_ARBITRUM = "0x92188c8200869b7bfB9A867C545ea723bD8AfEA1";

const GAS_PRICE_OSMOSIS = GasPrice.fromString("0.025uosmo");

const arbitrumClient = createPublicClient({
  chain: arbitrum,
  transport: http(process.env.EVM_RPC_URL),
});

let osmosisClient: SigningCosmWasmClient;
let osmosisSigner: DirectSecp256k1HdWallet;
let osmosisSignerAddress: string;

async function validateSolverBalances(inputCoins: Coin[]) {
  for (const inputCoin of inputCoins) {
    const balance = await osmosisClient.getBalance(
      osmosisSignerAddress,
      inputCoin.denom
    );

    if (BigInt(balance.amount) < BigInt(inputCoin.amount)) {
      throw new Error(
        `insufficient balance for ${inputCoin.denom} - expected: ${inputCoin.amount}, got: ${balance.amount}`
      );
    }
  }
}

async function handleOpenEvent(
  orderID: string,
  resolvedOrder: ResolvedCrossChainOrder
) {
  // this will only ever be 1 input (USDC on Osmosis)
  const inputCoins = resolvedOrder.maxSpent.map((output) => {
    // convert hex string to token denom
    const token = Buffer.from(output.token.replace("0x", ""), "hex").toString();
    return coin(output.amount.toString(), token);
  });

  await validateSolverBalances(inputCoins);

  // there is only ever one fill instruction
  const fillInstruction = resolvedOrder.fillInstructions[0];

  // the 7683 contract on Osmosis
  const destinationContract = toBech32(
    "osmo",
    fromHex(fillInstruction.destinationSettler.replace("0x", ""))
  );

  const msg = {
    typeUrl: MsgExecuteContract.typeUrl,
    value: MsgExecuteContract.fromPartial({
      sender: osmosisSignerAddress,
      contract: destinationContract,
      msg: Buffer.from(
        JSON.stringify({
          fill: {
            // annoying inconsistency between EVM and Cosmos
            order_id: orderID.replace("0x", ""),
            // convert from hex to base64
            origin_data: toBase64(
              fromHex(fillInstruction.originData.replace("0x", ""))
            ),
            filler_data: "",
          },
        })
      ),
      funds: inputCoins,
    }),
  };

  const estimatedGas = await osmosisClient.simulate(
    osmosisSignerAddress,
    [msg],
    undefined
  );

  const fee = calculateFee(
    parseInt((estimatedGas * 1.5).toFixed(0)),
    GAS_PRICE_OSMOSIS
  );

  const tx = await osmosisClient.signAndBroadcast(
    osmosisSignerAddress,
    [msg],
    fee,
    undefined
  );

  console.log("Tx Hash:", tx.transactionHash);
}

async function main() {
  const DEPLOYER_MNEMONIC = process.env.DEPLOYER_MNEMONIC;
  if (!DEPLOYER_MNEMONIC) {
    throw new Error("DEPLOYER_MNEMONIC is not set");
  }

  osmosisSigner = await DirectSecp256k1HdWallet.fromMnemonic(
    DEPLOYER_MNEMONIC,
    {
      prefix: "osmo",
    }
  );

  const accounts = await osmosisSigner.getAccounts();
  osmosisSignerAddress = accounts[0].address;

  osmosisClient = await SigningCosmWasmClient.connectWithSigner(
    "https://osmosis-rpc.polkachu.com",
    osmosisSigner
  );

  arbitrumClient.watchContractEvent({
    address: ERC7683_ADDRESS_ARBITRUM,
    abi: GOFAST_7683_ABI,
    eventName: "Open",
    onLogs: async (events) => {
      for (const event of events) {
        if (!event.args.orderID || !event.args.resolvedOrder) {
          continue;
        }

        await handleOpenEvent(event.args.orderID, event.args.resolvedOrder);
      }
    },
  });

  console.log("Watching for events...");
}

main().catch((err) => console.error(err));
