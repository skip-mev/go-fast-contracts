export interface Output {
  token: string;
  amount: bigint;
  recipient: `0x${string}`;
  chainId: bigint;
}

export interface FillInstruction {
  destinationChainId: bigint;
  destinationSettler: `0x${string}`;
  originData: `0x${string}`;
}

export interface ResolvedCrossChainOrder {
  user: `0x${string}`;
  originChainId: bigint;
  openDeadline: number;
  fillDeadline: number;
  maxSpent: readonly Output[];
  minReceived: readonly Output[];
  fillInstructions: readonly FillInstruction[];
}

export interface FastTransferOrder {
  sender: `0x${string}`;
  recipient: `0x${string}`;
  amountIn: bigint;
  amountOut: bigint;
  nonce: bigint;
  sourceDomain: number;
  destinationDomain: number;
  timeoutTimestamp: bigint;
  data: `0x${string}`;
}
