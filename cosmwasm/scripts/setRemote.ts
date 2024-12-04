import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { calculateFee, GasPrice } from "@cosmjs/stargate";
import { padHex } from "viem";
import { MsgExecuteContract } from "cosmjs-types/cosmwasm/wasm/v1/tx";

const GAS_PRICE = GasPrice.fromString("0.025uosmo");
const CHAIN_PREFIX = "osmo";
const RPC_URL = "https://osmosis-rpc.polkachu.com";

const DOMAIN = 42161;
const CONTRACT_ADDRESS = "0x23Cb6147E5600C23d1fb5543916D3D5457c9B54C";

interface MsgAddRemoteDomain {
  domain: number;
  address: string;
}

async function main() {
  const DEPLOYER_MNEMONIC = process.env.DEPLOYER_MNEMONIC;
  if (!DEPLOYER_MNEMONIC) {
    throw new Error("DEPLOYER_MNEMONIC is not set");
  }

  const signer = await DirectSecp256k1HdWallet.fromMnemonic(DEPLOYER_MNEMONIC, {
    prefix: CHAIN_PREFIX,
  });

  const accounts = await signer.getAccounts();
  const signerAddress = accounts[0].address;

  const client = await SigningCosmWasmClient.connectWithSigner(RPC_URL, signer);

  const formattedContractAddress = padHex(CONTRACT_ADDRESS, {
    dir: "left",
    size: 32,
  });

  const msgAddRemoteDomain: MsgAddRemoteDomain = {
    domain: DOMAIN,
    address: formattedContractAddress,
  };

  const msg = {
    typeUrl: MsgExecuteContract.typeUrl,
    value: MsgExecuteContract.fromPartial({
      sender: signerAddress,
      contract: CONTRACT_ADDRESS,
      msg: Buffer.from(
        JSON.stringify({
          add_remote_domain: msgAddRemoteDomain,
        })
      ),
    }),
  };

  const estimatedGas = await client.simulate(signerAddress, [msg], undefined);

  const fee = calculateFee(
    parseInt((estimatedGas * 1.5).toFixed(0)),
    GAS_PRICE
  );

  const tx = await client.signAndBroadcast(
    signerAddress,
    [msg],
    fee,
    undefined
  );

  console.log("Tx Hash:", tx.transactionHash);
}

main().catch(console.error);
