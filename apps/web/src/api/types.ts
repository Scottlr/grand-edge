export type ItemIcon = {
  sourceFileName: string;
  canonicalFileName: string;
  cdnUrl: string;
  source: "mapping_icon" | "html_source_match" | "missing";
};

export type Item = {
  itemId: number;
  name: string;
  examine: string | null;
  members: boolean;
  buyLimit: number | null;
  lowAlch: number | null;
  highAlch: number | null;
  value: number | null;
  icon: ItemIcon | null;
};

export type Interval = "5m" | "1h" | "6h" | "24h";

export type IntervalPrice = {
  itemId: number;
  bucketStart: string;
  interval: Interval;
  avgHighPrice: number | null;
  highPriceVolume: number;
  avgLowPrice: number | null;
  lowPriceVolume: number;
};

export type RecommendationAction = "buy" | "add" | "hold" | "cashout" | "avoid" | "watch";

export type StrategySignal = {
  itemId: number;
  strategyId: string;
  modelVersion: string;
  asOf: string;
  side: string;
  horizonSecs: number;
  confidence: number;
  expectedReturn: number;
  expectedNetGpPerUnit: number;
};

export type ScoreComponent = {
  key: string;
  label: string;
  value: number;
  weight: number | null;
};

export type RecommendationExplanation = {
  featureSetVersion: string;
  marketRulesVersion: string;
  strategyVotes: StrategySignal[];
  scoreComponents: ScoreComponent[];
  accuracySnapshot: unknown | null;
  structuredExplanation: unknown;
};

export type Recommendation = {
  recommendationId: string;
  userId: string | null;
  itemId: number;
  asOf: string;
  action: RecommendationAction;
  score: number;
  predictionConfidence: number | null;
  executionConfidence: number | null;
  recommendationConfidence: number;
  expectedNetGp: number | null;
  expectedRoi: number | null;
  riskLabel: string | null;
  reasons: string[];
  explanation: RecommendationExplanation;
};

export type StrategyStatus = {
  strategyId: string;
  enabled: boolean;
};

export type PatchStrategyRequest = {
  enabled: boolean;
};

export type Position = {
  positionId: string;
  userId: string;
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt: string | null;
  notes: string | null;
};

export type UpsertPositionRequest = {
  itemId: number;
  quantity: number;
  avgBuyPrice: number;
  boughtAt?: string | null;
  notes?: string | null;
};

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

export type LiveEvent =
  | {
      type: "price_updated";
      item_id: number;
      observed_at: string;
    }
  | {
      type: "recommendation_updated";
      recommendation_id: string;
      item_id: number;
      action: RecommendationAction;
    }
  | {
      type: "simulation_updated";
      run_id: string;
      status: string;
    }
  | {
      type: "strategy_config_updated";
      strategy_id: string;
      enabled: boolean;
    };
