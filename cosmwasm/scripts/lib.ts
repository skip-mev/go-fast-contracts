import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { GasPrice, StdFee, calculateFee, coin } from "@cosmjs/stargate";
import {
  MsgInstantiateContract,
  MsgInstantiateContractResponse,
  MsgMigrateContract,
  MsgMigrateContractResponse,
  MsgStoreCode,
  MsgStoreCodeResponse,
} from "cosmjs-types/cosmwasm/wasm/v1/tx";
import { MsgExec } from "cosmjs-types/cosmos/authz/v1beta1/tx";
import fs from "fs/promises";
import path from "path";

export async function storeContract(
  client: SigningCosmWasmClient,
  signerAddress: string,
  codePath: string,
  gasPrice: GasPrice
) {
  const byteCode = await fs.readFile(path.resolve(__dirname, codePath));

  // const msg = {
  //   typeUrl: MsgStoreCode.typeUrl,
  //   value: MsgStoreCode.fromPartial({
  //     sender: "osmo1raa4kyx5ypz75qqk3566c6slx2mw3qzsu6rymw",
  //     wasmByteCode: byteCode,
  //   }),
  // };

  const msg = {
    typeUrl: MsgExec.typeUrl,
    value: MsgExec.fromPartial({
      grantee: "osmo1zhqrfu9w3sugwykef3rq8t0vlxkz72vwd63q67",
      msgs: [
        {
          typeUrl: MsgStoreCode.typeUrl,
          value: MsgStoreCode.encode({
            sender: "osmo1raa4kyx5ypz75qqk3566c6slx2mw3qzsu6rymw",
            wasmByteCode: byteCode,
          }).finish(),
        },
      ],
    }),
  };

  const estimatedGas = await client.simulate(signerAddress, [msg], undefined);

  console.log(`Estimated gas: ${estimatedGas}`);

  const fee = calculateFee(parseInt((estimatedGas * 1.5).toFixed(0)), gasPrice);

  console.log(`Fee: ${fee}`);

  const tx = await client.signAndBroadcast(
    signerAddress,
    [msg],
    fee,
    undefined
  );

  const response = MsgStoreCodeResponse.decode(tx.msgResponses[0].value);

  return response;
}

export async function instantiateContract<V>(
  client: SigningCosmWasmClient,
  signerAddress: string,
  codeID: bigint,
  label: string,
  initMsg: V,
  gasPrice: GasPrice
) {
  const msg = {
    typeUrl: MsgInstantiateContract.typeUrl,
    value: MsgInstantiateContract.fromPartial({
      sender: signerAddress,
      admin: signerAddress,
      codeId: codeID,
      label: label,
      msg: Buffer.from(JSON.stringify(initMsg)),
    }),
  };

  const estimatedGas = await client.simulate(signerAddress, [msg], undefined);

  const fee = calculateFee(parseInt((estimatedGas * 1.5).toFixed(0)), gasPrice);

  const tx = await client.signAndBroadcast(
    signerAddress,
    [msg],
    fee,
    undefined
  );

  const response = MsgInstantiateContractResponse.decode(
    tx.msgResponses[0].value
  );

  return response;
}

export async function migrateContract<V>(
  client: SigningCosmWasmClient,
  signerAddress: string,
  contractAddress: string,
  codeID: bigint,
  gasPrice: GasPrice
) {
  const msg = {
    typeUrl: MsgMigrateContract.typeUrl,
    value: MsgMigrateContract.fromPartial({
      sender: signerAddress,
      contract: contractAddress,
      codeId: codeID,
      // label: label,
      msg: Buffer.from(JSON.stringify({})),
    }),
  };

  const estimatedGas = await client.simulate(signerAddress, [msg], undefined);

  const fee = calculateFee(parseInt((estimatedGas * 1.5).toFixed(0)), gasPrice);

  const tx = await client.signAndBroadcast(
    signerAddress,
    [msg],
    fee,
    undefined
  );

  const response = MsgMigrateContractResponse.decode(tx.msgResponses[0].value);

  return response;
}
