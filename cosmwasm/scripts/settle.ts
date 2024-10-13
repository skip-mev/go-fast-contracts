import "dotenv/config";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { GasPrice } from "@cosmjs/stargate";
import { instantiateContract, storeContract } from "./lib";

const GAS_PRICE = GasPrice.fromString("0.0053untrn");
const CHAIN_PREFIX = "neutron";
const RPC_URL = "https://neutron-rpc.polkachu.com";

const tokenDenom =
  "ibc/B559A80D62249C8AA07A380E2A2BEA6E5CA9A6F079C912C3A9E9B494105E4F81";
const addressPrefix = "neutron";
const mailboxAddr =
  "neutron1sjzzd4gwkggy6hrrs8kxxatexzcuz3jecsxm3wqgregkulzj8r7qlnuef4";
const remoteDomain = 42161;
const remoteAddr =
  "000000000000000000000000f7b86fDee755f1821A6A7467ebf75A2BF7Aea227";

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

  const client = await SigningCosmWasmClient.connectWithSigner(RPC_URL, signer);

  const fillMessage = {
    initiate_settlement: {
      order_id:
        "23FF50A09046CABF27EBC1F5527F9540F149F80D4AE53112DC4414A0DE646DA8",
      repayment_address:
        "00000000000000000000000056Ca414d41CD3C1188A4939b0D56417dA7Bb6DA2",
    },
  };

  const hyperlaneFee = {
    denom:
      "ibc/773B4D0A3CD667B2275D5A4A7A2F0909C0BA0F4059C0B9181E680DDF4965DCC7",
    amount: "270000",
  };

  const gasFee = { denom: "untrn", amount: "5300" };

  const result = await client.execute(
    signerAddress,
    "neutron13peucvly3gtd66cyx5jl9at9ursaclmv6ws2e07xh8wx23x8xqxq8dtpxr",
    fillMessage,
    {
      amount: [gasFee],
      gas: "1000000",
    },
    undefined,
    [hyperlaneFee]
  );

  console.log(result);
  console.log(result.transactionHash);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
