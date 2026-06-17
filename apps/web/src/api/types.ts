export type {
  AuthenticatedUser,
  ExecutionMode,
  LoginRequest,
  RegisterRequest,
  RiskProfile,
  UpdateRiskProfileRequest,
} from "../domain/auth";
export type { Item, ItemIcon } from "../domain/item";
export type {
  DataState,
  InvalidationRuleDto,
  MarketStatusDto,
  ModelAccuracySummaryDto,
  RecommendationAction,
  RecommendationDto as Recommendation,
  RecommendationViewModel,
  ScoreComponent,
} from "../domain/recommendation";
export type {
  Position,
  UpsertPositionRequest,
} from "../domain/portfolio";
export type {
  CreateSimulationRequest,
  SimulationRun,
} from "../domain/simulation";
export type {
  ExecutionConfidenceDto,
  PatchStrategyRequest,
  StrategyStatus,
  StrategyVoteDto as StrategySignal,
} from "../domain/strategy";
export type {
  RecommendationEvidence,
  EvidenceStage,
  ModelCardRef,
} from "../domain/evidence";

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
      action: import("../domain/recommendation").RecommendationAction;
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
