export type TimePoint = {
  timestamp: string;
  label: string;
  mid: number | null;
  high: number | null;
  low: number | null;
  spread: number | null;
  volume: number;
};

export type PricePoint = {
  timestamp: string;
  label: string;
  mid: number | null;
  high: number | null;
  low: number | null;
  volume: number | null;
  spread: number | null;
};

export type ForecastBandPoint = {
  timestamp: string;
  label: string;
  lower: number | null;
  predicted: number | null;
  upper: number | null;
};

export type DrawdownPoint = {
  label: string;
  value: number | null;
  status: "realized" | "open" | "skipped";
};

export type CalibrationBucket = {
  label: string;
  predicted: number;
  actual: number | null;
  sampleSize: number;
};

export type RecommendationMarkers = {
  entry: number | null;
  exit: number | null;
  stopLoss: number | null;
};

export type ChartLayer =
  | "mid"
  | "forecastBand"
  | "entry"
  | "exit"
  | "stopLoss"
  | "volume"
  | "spread"
  | "strategyVotes"
  | "regime"
  | "simulationTrades";

export const defaultChartLayerLabels: Record<ChartLayer, string> = {
  mid: "Current price",
  forecastBand: "Likely price range",
  entry: "Suggested buy",
  exit: "Suggested sell",
  stopLoss: "Danger point",
  volume: "Observed activity",
  spread: "Spread",
  strategyVotes: "Advanced method views",
  regime: "Market mood",
  simulationTrades: "Past test trades",
};

export const technicalChartTermsHiddenByDefault = [
  "entry",
  "exit",
  "stop-loss",
  "confidence interval",
  "drawdown",
  "regime",
  "liquidity proxy",
] as const;
