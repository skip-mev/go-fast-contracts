# How to deploy the Go Fast contracts

## Deploying the Cosmwasm contract

### 1. Create a `.env` file in the `cosmwasm` directory with the following:

```
DEPLOYER_MNEMONIC=<your_mnemonic>
```

### 2. Store the contract:

```
npx ts-node scripts/storeContract.ts
```

### 3. Instantiate the contract:

First, update the variables in `scripts/instantiateContract.ts` to match your configuration.

Then, run the following command:

```
npx ts-node scripts/instantiateContract.ts
```

This script will output the contract address, as well as the hex representation of the contract address. Save this hex address for use later.

## Deploying the Solidity contract

### 1. Update `Deploy.s.sol` configuration.

Most of the variables in `Deploy.s.sol` are constants and can be left as is. The only variable that you will need to update is `owner`.

### 2. Deploy the contract.

```
forge script ./script/Deploy.s.sol --rpc-url <RPC_URL> --private-key <PRIVATE_KEY> --broadcast
```

## Setting the remote contract (EVM)

In order to submit and settle orders you must make the Solidity contract aware of the Cosmwasm deployment.

### 1. Update `SetRemote.s.sol` configuration.

In `script/SetRemote.s.sol`, update the `remoteContract` variable with the hex address that was logged when the Cosmwasm contract was instantiated. (You'll have to remove the `0x` prefix from the address.)

### 2. Set the remote contract.

```
forge script ./script/SetRemote.s.sol --rpc-url <RPC_URL> --private-key <PRIVATE_KEY> --broadcast
```

## Setting the remote contract (Cosmwasm)

In order to fill orders and request settlements you must make the Cosmwasm contract aware of the Solidity contract.

### 1. Update `setRemote.ts` configuration.

In `cosmwasm/scripts/setRemote.ts`, update the `CONTRACT_ADDRESS` variable with the address of the deployed Solidity contract.

### 2. Set the remote contract.

```
npx ts-node cosmwasm/scripts/setRemote.ts
```
