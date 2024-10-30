import { $ } from "bun";

async function getOrderIDFromSolidity() {
  const response = await $`forge script ./script/PrintOrderID.s.sol`
    .cwd("../solidity")
    .quiet();

  const parts = response.text().split("== Logs ==");

  return parts[1].trim();
}

async function getOrderIDFromCosmwasm() {
  const response = await $`cargo run ./bin/print-order-id`
    .cwd("../cosmwasm")
    .quiet();

  const parts = response.text().split("== Output ==");

  return `0x${parts[1].trim()}`;
}

async function main() {
  const solidityOrderID = await getOrderIDFromSolidity();
  const cosmwasmOrderID = await getOrderIDFromCosmwasm();

  if (!solidityOrderID || !cosmwasmOrderID) {
    throw new Error("Failed to get order IDs");
  }

  if (solidityOrderID !== cosmwasmOrderID) {
    throw new Error(
      `Order IDs do not match\nSolidity: ${solidityOrderID}\nCosmwasm: ${cosmwasmOrderID}`
    );
  }

  console.log("Order IDs match");
}

main();
