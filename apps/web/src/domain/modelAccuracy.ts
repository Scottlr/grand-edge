import type { CalibrationBucket } from "../charts/chartTypes";

export type AccuracyStatus = "well_calibrated" | "under_confident" | "over_confident" | "insufficient_sample" | "unavailable";

export type ModelAccuracyViewModel = {
  strategyId: string;
  modelVersion: string;
  windowLabel: "7d" | "30d" | "all";
  sampleSize: number;
  directionalAccuracy: number | null;
  brierScore: number | null;
  netSimulatedGp: number | null;
  winRate: number | null;
  profitFactor: number | null;
  maxDrawdown: number | null;
  avgTimeInTrade: string | null;
  capitalEfficiency: number | null;
  calibrationBuckets: CalibrationBucket[];
  whatItDoes: string;
  worksBestWhen: string;
  weakness: string;
};

export type TrustSummaryViewModel = {
  buyCallsProfitable: number | null;
  sellCallsProtectedProfit: number | null;
  averageProfitPerSuccessfulCall: number | null;
  bestMethodThisWeek: string | null;
  weakestMethodThisWeek: string | null;
  confidenceHonestySentence: string | null;
};

export function accuracyStatusForModel(model: ModelAccuracyViewModel): AccuracyStatus {
  if (model.sampleSize < 10) {
    return "insufficient_sample";
  }
  if (model.directionalAccuracy === null || model.calibrationBuckets.length === 0) {
    return "unavailable";
  }

  const pairedBuckets = model.calibrationBuckets.filter((bucket) => bucket.actual !== null);
  if (pairedBuckets.length === 0) {
    return "unavailable";
  }

  const averageGap =
    pairedBuckets.reduce((sum, bucket) => sum + Math.abs(bucket.predicted - (bucket.actual ?? 0)), 0) / pairedBuckets.length;
  const actualMean =
    pairedBuckets.reduce((sum, bucket) => sum + (bucket.actual ?? 0), 0) / pairedBuckets.length;
  const predictedMean = pairedBuckets.reduce((sum, bucket) => sum + bucket.predicted, 0) / pairedBuckets.length;

  if (averageGap <= 0.06) {
    return "well_calibrated";
  }
  if (predictedMean > actualMean) {
    return "over_confident";
  }

  return "under_confident";
}
