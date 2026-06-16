export type StrategyStatus = {
  strategyId: string;
  enabled: boolean;
};

export type PatchStrategyRequest = {
  enabled: boolean;
};

export type ExecutionConfidenceDto = {
  observedVolume: number;
  observedVolumeZ: number | null;
  estimatedFillProbability: number | null;
  estimatedCapacity: number | null;
  liquidityConfidence: number | null;
  note: string;
};

export type StrategyVoteDto = {
  itemId: number;
  strategyId: string;
  modelVersion: string;
  asOf: string;
  side: string;
  horizonSecs: number;
  confidence: number;
  expectedReturn: number;
  expectedNetGpPerUnit: number;
  targetEntry: number | null;
  targetExit: number | null;
  stopLoss: number | null;
  takeProfit: number | null;
  maxQuantity: number | null;
  execution: ExecutionConfidenceDto | null;
  explanation: unknown;
};
