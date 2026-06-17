export type SimulationRun = {
  runId: string;
  name: string;
  strategyConfig: unknown;
  startedAt: string;
  finishedAt: string | null;
  status: string;
};

export type CreateSimulationRequest = {
  name: string;
  strategyConfig?: unknown;
};

export type SimulationModeLabel = "Safe test" | "Normal test" | "Best-case test";

export const simulationModeAdvancedLabels: Record<SimulationModeLabel, string> = {
  "Safe test": "Conservative execution",
  "Normal test": "Balanced execution",
  "Best-case test": "Optimistic execution",
};

export type PaperBetHitReason =
  | "target_exit"
  | "stop_loss"
  | "horizon_expired"
  | "manual_cashout"
  | "open"
  | "skipped";

export type PaperBetView = {
  betId: string;
  strategyId: string;
  itemId: number;
  itemName: string;
  entryTime: string;
  entryPrice: number;
  quantity: number;
  targetExit: number | null;
  stopLoss: number | null;
  exitTime: string | null;
  exitPrice: number | null;
  taxPaid: number;
  expectedNetGp: number | null;
  realizedProfitGp: number | null;
  realizedRoi: number | null;
  maxDrawdown: number | null;
  hitReason: PaperBetHitReason;
  confidenceAtEntry: number | null;
  modeLabel: SimulationModeLabel;
  slippageEstimateGp: number | null;
};
