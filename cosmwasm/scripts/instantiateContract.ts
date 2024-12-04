import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { fromBech32 } from "@cosmjs/encoding";
import { instantiateContract } from "./lib";

const GAS_PRICE = GasPrice.fromString("0.025uosmo");
const CHAIN_PREFIX = "osmo";
const RPC_URL = "https://osmosis-rpc.polkachu.com";

const CODE_ID = 1000;

const tokenDenom =
  "ibc/498A0751C798A0D9A389AA3691123DADA57DAA4FE165D5C75894505B876BA6E4";
const mailboxAddr =
  "osmo1jjf788v9m5pcqghe0ky2hf4llxxe37dqz6609eychuwe3xzzq9eql969h3";
const hookAddr =
  "osmo13yswqchwtmv2ln9uz4w3865sfy5k8x0wg9qrv4vxflxjg0kuwwyqqpvqxz";
const localDomain = 875;

interface InstantiateMsg {
  token_denom: string;
  address_prefix: string;
  mailbox_addr: string;
  hook_addr: string;
  local_domain: number;
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

  const initMsg: InstantiateMsg = {
    token_denom: tokenDenom,
    address_prefix: CHAIN_PREFIX,
    mailbox_addr: mailboxAddr,
    hook_addr: hookAddr,
    local_domain: localDomain,
  };

  const { address } = await instantiateContract(
    client,
    signerAddress,
    BigInt(CODE_ID),
    "fast-transfer",
    initMsg,
    GAS_PRICE
  );

  console.log(`Contract Address: ${address}`);
  console.log(`Hex: ${Buffer.from(fromBech32(address).data).toString("hex")}`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
