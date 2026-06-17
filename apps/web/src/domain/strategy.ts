import type { DataState } from "./recommendation";

export type StrategyStatus = {
  strategyId: string;
  enabled: boolean;
};

export type PatchStrategyRequest = {
  enabled: boolean;
};

export type StrategyStatusLevel = "active" | "disabled" | "degraded" | "insufficient_data" | "error";

export type StrategyPaperBet = {
  itemName: string;
  actionLabel: "BUY" | "SELL" | "WAIT";
  outcomeLabel: string;
  netGp: number | null;
  confidence: number | null;
};

export type StrategyLabRow = {
  strategyId: string;
  displayName: string;
  enabled: boolean;
  weight: number | null;
  netGp30d: number | null;
  accuracy30d: number | null;
  currentConfidence: number | null;
  status: StrategyStatusLevel;
  bestRegime: string | null;
  worstRegime: string | null;
  lastUpdatedAt: string | null;
  canToggle: boolean;
  disabledReason: string | null;
};

export type StrategyLabDetail = {
  strategyId: string;
  displayName: string;
  summary: string;
  whatItLooksFor: string;
  currentWeightLabel: string | null;
  recentPerformanceLabel: string | null;
  bestRegime: string | null;
  worstRegime: string | null;
  currentConfidenceLabel: string | null;
  paperBets: StrategyPaperBet[];
  configSummary: string[];
  advanced: {
    strategyId: string;
    modelVersion: string | null;
    configJson: string;
    statusNote: string | null;
  };
};

export type StrategyLabViewModel = {
  rows: StrategyLabRow[];
  selectedStrategyId: string | null;
  dataState: DataState;
  staleReason: string | null;
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

export type StrategyFixtureCatalog = Record<
  string,
  Omit<StrategyLabRow, "enabled"> & {
    detail: StrategyLabDetail;
  }
>;

export function toStrategyLabViewModel(
  dto: StrategyStatus[],
  fixtures: StrategyFixtureCatalog,
  options?: {
    dataState?: DataState;
    staleReason?: string | null;
    selectedStrategyId?: string | null;
  },
): StrategyLabViewModel {
  const rows = dto.map((entry) => {
    const fixture = fixtures[entry.strategyId];
    if (!fixture) {
      return {
        strategyId: entry.strategyId,
        displayName: entry.strategyId,
        enabled: entry.enabled,
        weight: null,
        netGp30d: null,
        accuracy30d: null,
        currentConfidence: null,
        status: "error" as const,
        bestRegime: null,
        worstRegime: null,
        lastUpdatedAt: null,
        canToggle: false,
        disabledReason: "This method is not recognized by the current frontend contract yet.",
      };
    }

    return {
      ...fixture,
      enabled: entry.enabled,
      status: entry.enabled ? fixture.status : "disabled",
      canToggle: fixture.canToggle,
      disabledReason: entry.enabled ? fixture.disabledReason : fixture.disabledReason ?? "This method is currently turned off.",
    };
  });

  return {
    rows,
    selectedStrategyId: options?.selectedStrategyId ?? rows[0]?.strategyId ?? null,
    dataState: options?.dataState ?? "live",
    staleReason: options?.staleReason ?? null,
  };
}

export function formatStrategyPercent(value: number | null): string {
  if (value === null) {
    return "Not enough data yet";
  }

  return `${Math.round(value * 100)}%`;
}

export function formatStrategyGp(value: number | null): string {
  if (value === null) {
    return "Not enough data yet";
  }

  return `${value} gp`;
}

export function formatStrategyWeight(value: number | null): string {
  if (value === null) {
    return "Not enough data yet";
  }

  return `${Math.round(value * 100)}%`;
}
