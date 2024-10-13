import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { fromBech32 } from "@cosmjs/encoding";
import { instantiateContract, migrateContract, storeContract } from "./lib";

const GAS_PRICE = GasPrice.fromString("0.025uosmo");
const CHAIN_PREFIX = "osmo";
const RPC_URL = "https://osmosis-rpc.polkachu.com";

const CONTRACT_ADDRESS =
  "osmo1cnze5c4y7jw69ghzczsnu9v9qz3xuvevw5ayr2g0pa3ayafumlusej3pf5";

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
    "../artifacts/go_fast_transfer_cw-aarch64.wasm",
    GAS_PRICE
  );

  await migrateContract(
    client,
    signerAddress,
    CONTRACT_ADDRESS,
    codeId,
    GAS_PRICE
  );
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
