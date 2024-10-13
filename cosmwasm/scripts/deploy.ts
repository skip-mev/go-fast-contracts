import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { fromBech32 } from "@cosmjs/encoding";
import { instantiateContract, storeContract } from "./lib";

const GAS_PRICE = GasPrice.fromString("0.025uosmo");
const CHAIN_PREFIX = "osmo";
const RPC_URL = "https://osmosis-rpc.polkachu.com";

const tokenDenom =
  "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4";
const mailboxAddr =
  "osmo1jjf788v9m5pcqghe0ky2hf4llxxe37dqz6609eychuwe3xzzq9eql969h3";
const remoteDomain = 42161;
const remoteAddr =
  "000000000000000000000000F7ceC3d387384bB6cE5792dAb161a65cFaCf8aB4";

interface InstantiateMsg {
  token_denom: string;
  address_prefix: string;
  mailbox_addr: string;
  remote_domain: number;
  remote_addr: string;
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

  // await storeContract(
  //   client,
  //   signerAddress,
  //   "../artifacts/go_fast_transfer_cw-aarch64.wasm",
  //   GAS_PRICE
  // );

  const codeId = BigInt(1000);

  // console.log(`Code ID: ${codeId}`);

  const initMsg: InstantiateMsg = {
    token_denom: tokenDenom,
    address_prefix: CHAIN_PREFIX,
    mailbox_addr: mailboxAddr,
    remote_domain: remoteDomain,
    remote_addr: remoteAddr,
  };

  const { address } = await instantiateContract(
    client,
    signerAddress,
    codeId,
    "fast-transfer",
    initMsg,
    GAS_PRICE
  );

  console.log(`Contract Address: ${address}`);
  console.log(Buffer.from(fromBech32(address).data).toString("hex"));
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
