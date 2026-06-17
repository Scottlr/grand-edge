import type { RecommendationDto } from "./recommendation";

export type EvidenceStageKind =
  | "market_data"
  | "feature_snapshot"
  | "graph_context"
  | "prediction"
  | "recommendation"
  | "explanation"
  | "outcome_evaluation";

export type EvidenceStageStatus = "present" | "pending" | "degraded" | "missing";
export type EvidenceDataStateStatus = "live" | "pending" | "stale" | "degraded" | "empty" | "error";

export type EvidenceStage = {
  kind: EvidenceStageKind;
  label: string;
  timestamp: string | null;
  status: EvidenceStageStatus;
};

export type FeatureSnapshot = {
  featureSnapshotId: string;
  itemId: number;
  asOf: string;
  featureSetVersion: string;
  graphVersion: string | null;
  sourceWindowStart: string;
  sourceWindowEnd: string;
  features: Record<string, unknown>;
};

export type PredictionEvidence = {
  predictionId: string;
  featureSnapshotId: string;
  itemId: number;
  asOf: string;
  horizonSecs: number;
  modelId: string;
  modelVersion: string;
  predictedDirection: string;
  predictedReturn: number | null;
  confidence: number;
  predictionIntervalLow: number | null;
  predictionIntervalHigh: number | null;
  explanation: unknown;
};

export type PredictionLink = {
  predictionId: string;
  contributionWeight: number;
  modelId: string;
  modelVersion: string;
};

export type ReasonAtom = {
  reasonType: string;
  reasonKey: string;
  label: string;
  direction: string;
  weight: number;
  evidence: unknown;
};

export type StructuredExplanation = {
  summary: string;
  reasonAtoms: ReasonAtom[];
  invalidationRules: RecommendationDto["invalidationRules"];
  graphVersion: string | null;
  graphReasonPathCount: number | null;
};

export type RecommendationOutcome = {
  evaluatedAt: string;
  horizonSecs: number;
  actualReturn: number | null;
  actualNetGp: number | null;
  directionCorrect: boolean | null;
  hitTakeProfit: boolean;
  hitStopLoss: boolean;
  outcomeLabel: string;
};

export type ReasonPerformance = {
  reasonType: string;
  reasonKey: string;
  modelVersion: string;
  sampleSize: number;
  winRate: number | null;
  avgActualReturn: number | null;
  avgNetGp: number | null;
  calibrationError: number | null;
};

export type ModelCardRef = {
  modelId: string;
  modelVersion: string;
  artifactHash: string | null;
  featureSchemaHash: string | null;
};

export type GraphPath = {
  sourceItemId: number;
  targetItemId: number;
  relationType: string;
  edgeId: string | null;
  eventId: string | null;
  contributionWeight: number | null;
  explanation: unknown;
};

export type GraphSource = {
  relationType: string;
  sourceItemId: number;
  targetItemId: number;
  contributionWeight: number | null;
};

export type RecommendationEvidence = {
  recommendationId: string;
  itemId: number;
  asOf: string;
  stages: EvidenceStage[];
  featureSnapshot: FeatureSnapshot | null;
  predictions: PredictionEvidence[];
  predictionLinks: PredictionLink[];
  recommendation: RecommendationDto;
  graphVersion: string | null;
  graphPaths: GraphPath[];
  graphSources: GraphSource[];
  explanation: StructuredExplanation;
  outcome: RecommendationOutcome | null;
  reasonPerformance: ReasonPerformance[];
  modelCards: ModelCardRef[];
  dataState: {
    status: EvidenceDataStateStatus;
    reason: string | null;
  };
};
