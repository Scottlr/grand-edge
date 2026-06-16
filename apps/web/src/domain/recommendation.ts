import type { ItemIcon } from "./item";
import type { StrategyVoteDto } from "./strategy";

export type DataState = "loading" | "live" | "stale" | "degraded" | "empty" | "error";
export type RecommendationAction = "buy" | "add" | "hold" | "cashout" | "avoid" | "watch";
export type RiskLabel = "low" | "medium" | "high" | "unknown";

export type ScoreComponent = {
  key: string;
  label: string;
  value: number;
  weight: number | null;
};

export type InvalidationRuleDto = {
  metric: string;
  operator: string;
  threshold: string;
  currentValue: string | null;
  reason: string;
};

export type ModelAccuracySummaryDto = {
  strategyId: string;
  modelVersion: string;
  lookbackWindow: string;
  sampleSize: number;
  directionalAccuracy: number | null;
  brierScore: number | null;
  avgRealizedRoi: number | null;
  maxDrawdown: number | null;
  calibration: unknown;
};

export type ConfidenceBreakdownDto = {
  confidence: number;
  predictionConfidence: number | null;
  executionConfidence: number | null;
  recommendationConfidence: number;
  modelAgreementLabel: string;
  recentAccuracy: number | null;
  dataQualityLabel: string;
  executionQualityLabel: string | null;
  regimeLabel: string | null;
  penalties: ScoreComponent[];
};

export type MarketStatusDto = {
  dataState: DataState;
  staleReason: string | null;
  degradedReason: string | null;
};

export type RecommendationDto = {
  recommendationId: string;
  userId: string | null;
  itemId: number;
  itemName: string;
  itemIcon: ItemIcon | null;
  asOf: string;
  action: RecommendationAction;
  score: number;
  confidence: number;
  predictionConfidence: number | null;
  executionConfidence: number | null;
  recommendationConfidence: number;
  execution: import("./strategy").ExecutionConfidenceDto | null;
  expectedNetGp: number | null;
  expectedRoi: number | null;
  riskLabel: string | null;
  horizonSeconds: number;
  primaryReason: string;
  reasons: string[];
  invalidationRules: InvalidationRuleDto[];
  modelAgreement: number;
  confidenceBreakdown: ConfidenceBreakdownDto;
  strategyVotes: StrategyVoteDto[];
  accuracy: ModelAccuracySummaryDto | null;
  dataState: DataState;
  marketStatus: MarketStatusDto;
};

export type RecommendationViewModel = {
  recommendationId: string;
  itemId: number;
  itemName: string;
  itemIcon: ItemIcon | null;
  action: RecommendationAction;
  score: number;
  confidence: number;
  dataState: DataState;
  primaryReason: string;
  expectedNetGp: number | null;
  expectedRoi: number | null;
  strategyVotes: StrategyVoteDto[];
};

export function normalizeRiskLabel(label: string | null): RiskLabel {
  if (label === "low" || label === "medium" || label === "high") {
    return label;
  }

  return "unknown";
}

export function toRecommendationViewModel(dto: RecommendationDto): RecommendationViewModel {
  return {
    recommendationId: dto.recommendationId,
    itemId: dto.itemId,
    itemName: dto.itemName,
    itemIcon: dto.itemIcon,
    action: dto.action,
    score: dto.score,
    confidence: dto.confidence,
    dataState: dto.dataState,
    primaryReason: dto.primaryReason,
    expectedNetGp: dto.expectedNetGp,
    expectedRoi: dto.expectedRoi,
    strategyVotes: dto.strategyVotes,
  };
}

const baseRecommendation: RecommendationDto = {
  recommendationId: "rec-live",
  userId: null,
  itemId: 4151,
  itemName: "Abyssal whip",
  itemIcon: {
    sourceFileName: "Abyssal whip.png",
    canonicalFileName: "Abyssal_whip.png",
    cdnUrl: "https://oldschool.runescape.wiki/images/Abyssal_whip.png",
    source: "mapping_icon",
  },
  asOf: "2026-06-16T12:00:00Z",
  action: "buy",
  score: 0.81,
  confidence: 0.78,
  predictionConfidence: 0.8,
  executionConfidence: 0.7,
  recommendationConfidence: 0.78,
  execution: {
    observedVolume: 400,
    observedVolumeZ: 1.2,
    estimatedFillProbability: 0.64,
    estimatedCapacity: 8,
    liquidityConfidence: 0.7,
    note: "Observed volume is a proxy, not true GE depth.",
  },
  expectedNetGp: 1400,
  expectedRoi: 0.03,
  riskLabel: "low",
  horizonSeconds: 3600,
  primaryReason: "Tax-adjusted edge clears threshold.",
  reasons: ["Tax-adjusted edge clears threshold."],
  invalidationRules: [
    {
      metric: "final_score",
      operator: "<",
      threshold: "0.05",
      currentValue: "0.81",
      reason: "Score threshold",
    },
  ],
  modelAgreement: 1,
  confidenceBreakdown: {
    confidence: 0.78,
    predictionConfidence: 0.8,
    executionConfidence: 0.7,
    recommendationConfidence: 0.78,
    modelAgreementLabel: "high agreement",
    recentAccuracy: 0.66,
    dataQualityLabel: "live",
    executionQualityLabel: "strong",
    regimeLabel: null,
    penalties: [],
  },
  strategyVotes: [
    {
      itemId: 4151,
      strategyId: "spread_edge_v1",
      modelVersion: "v1",
      asOf: "2026-06-16T12:00:00Z",
      side: "buy",
      horizonSecs: 3600,
      confidence: 0.8,
      expectedReturn: 0.03,
      expectedNetGpPerUnit: 1400,
      targetEntry: 100000,
      targetExit: 104000,
      stopLoss: 99000,
      takeProfit: 104000,
      maxQuantity: 8,
      execution: {
        observedVolume: 400,
        observedVolumeZ: 1.2,
        estimatedFillProbability: 0.64,
        estimatedCapacity: 8,
        liquidityConfidence: 0.7,
        note: "Observed volume is a proxy, not true GE depth.",
      },
      explanation: { reason: "fixture" },
    },
  ],
  accuracy: {
    strategyId: "spread_edge_v1",
    modelVersion: "v1",
    lookbackWindow: "seven_days",
    sampleSize: 12,
    directionalAccuracy: 0.66,
    brierScore: 0.18,
    avgRealizedRoi: 0.02,
    maxDrawdown: 0.1,
    calibration: {},
  },
  dataState: "live",
  marketStatus: {
    dataState: "live",
    staleReason: null,
    degradedReason: null,
  },
};

export const recommendationMocks: Record<DataState, RecommendationDto> = {
  live: baseRecommendation,
  stale: {
    ...baseRecommendation,
    recommendationId: "rec-stale",
    dataState: "stale",
    marketStatus: {
      dataState: "stale",
      staleReason: "Recommendation evidence is based on stale market data.",
      degradedReason: null,
    },
    confidenceBreakdown: {
      ...baseRecommendation.confidenceBreakdown,
      dataQualityLabel: "stale",
    },
  },
  degraded: {
    ...baseRecommendation,
    recommendationId: "rec-degraded",
    dataState: "degraded",
    marketStatus: {
      dataState: "degraded",
      staleReason: null,
      degradedReason: "Recommendation evidence is incomplete.",
    },
    confidenceBreakdown: {
      ...baseRecommendation.confidenceBreakdown,
      dataQualityLabel: "degraded",
      executionQualityLabel: "uncertain",
    },
  },
  empty: {
    ...baseRecommendation,
    recommendationId: "rec-empty",
    dataState: "empty",
    reasons: [],
    primaryReason: "No recommendation evidence is available yet.",
    marketStatus: {
      dataState: "empty",
      staleReason: null,
      degradedReason: "Recommendation explanation is empty.",
    },
  },
  error: {
    ...baseRecommendation,
    recommendationId: "rec-error",
    dataState: "error",
    marketStatus: {
      dataState: "error",
      staleReason: null,
      degradedReason: "Backend request failed.",
    },
  },
  loading: {
    ...baseRecommendation,
    recommendationId: "rec-loading",
    dataState: "loading",
    marketStatus: {
      dataState: "loading",
      staleReason: null,
      degradedReason: null,
    },
  },
};

export function recommendationMocksCoverAllDataStates(): boolean {
  const expectedStates: DataState[] = ["loading", "live", "stale", "degraded", "empty", "error"];
  const actualStates = new Set(
    Object.values(recommendationMocks).map((recommendation) => recommendation.dataState),
  );

  return expectedStates.every((state) => actualStates.has(state));
}
