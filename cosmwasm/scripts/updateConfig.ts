import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { calculateFee, GasPrice } from "@cosmjs/stargate";
import { padHex } from "viem";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { fromBech32 } from "@cosmjs/encoding";

const GAS_PRICE = GasPrice.fromString("0.025uosmo");
const CHAIN_PREFIX = "osmo";
const RPC_URL = "https://osmosis-rpc.polkachu.com";

const CONTRACT_ADDRESS =
  "osmo19a2a86dtmlrngnjs6f0trrk3sfr9hf75n6yvlpplx6rp7z9mq4wqgmxwpm";

async function main() {
  // const DEPLOYER_MNEMONIC = process.env.DEPLOYER_MNEMONIC;
  // if (!DEPLOYER_MNEMONIC) {
  //   throw new Error("DEPLOYER_MNEMONIC is not set");
  // }
  // const signer = await DirectSecp256k1HdWallet.fromMnemonic(DEPLOYER_MNEMONIC, {
  //   prefix: CHAIN_PREFIX,
  // });
  // const accounts = await signer.getAccounts();
  // const signerAddress = accounts[0].address;
  // const client = await SigningCosmWasmClient.connectWithSigner(RPC_URL, signer);
  // const config = await client.queryContractSmart(CONTRACT_ADDRESS, {
  //   config: {},
  // });
  // const remoteAddr = padHex("0xF6c6f08705A95fb0ED33439e2dbA69Ba81AAA5C3", {
  //   dir: "left",
  //   size: 32,
  // });
  // const msg = {
  //   typeUrl: MsgExecuteContract.typeUrl,
  //   value: MsgExecuteContract.fromPartial({
  //     sender: signerAddress,
  //     contract: CONTRACT_ADDRESS,
  //     msg: Buffer.from(
  //       JSON.stringify({
  //         update_config: {
  //           config: {
  //             ...config,
  //             remote_addr: remoteAddr.replace("0x", ""),
  //           },
  //         },
  //       })
  //     ),
  //   }),
  // };
  // const estimatedGas = await client.simulate(signerAddress, [msg], undefined);
  // const fee = calculateFee(
  //   parseInt((estimatedGas * 1.5).toFixed(0)),
  //   GAS_PRICE
  // );
  // const tx = await client.signAndBroadcast(
  //   signerAddress,
  //   [msg],
  //   fee,
  //   undefined
  // );
  // console.log("Tx Hash:", tx.transactionHash);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
