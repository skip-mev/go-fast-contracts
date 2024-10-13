import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { fromBech32 } from "@cosmjs/encoding";
import { instantiateContract, storeContract } from "./lib";

const GAS_PRICE = GasPrice.fromString("0.025uosmo");
const CHAIN_PREFIX = "osmo";
const RPC_URL = "https://osmosis-rpc.polkachu.com";

const GATEWAY_ADDRESS =
  "osmo1cnze5c4y7jw69ghzczsnu9v9qz3xuvevw5ayr2g0pa3ayafumlusej3pf5";

const tokenDenom =
  "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4";
const mailboxAddr =
  "osmo1jjf788v9m5pcqghe0ky2hf4llxxe37dqz6609eychuwe3xzzq9eql969h3";
const remoteDomain = 42161;
const remoteAddr =
  "000000000000000000000000F7ceC3d387384bB6cE5792dAb161a65cFaCf8aB4";

interface InstantiateMsg {
  gateway_address: string;
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

  console.log(`Signer Address: ${signerAddress}`);

  const client = await SigningCosmWasmClient.connectWithSigner(RPC_URL, signer);

  const { codeId } = await storeContract(
    client,
    signerAddress,
    "../artifacts/cw_7683-aarch64.wasm",
    GAS_PRICE
  );

  console.log(`Code ID: ${codeId}`);

  const initMsg: InstantiateMsg = {
    gateway_address: GATEWAY_ADDRESS,
  };

  const { address } = await instantiateContract(
    client,
    signerAddress,
    codeId,
    "cw7683",
    initMsg,
    GAS_PRICE
  );

  console.log(`Contract Address: ${address}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
