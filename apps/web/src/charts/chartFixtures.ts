import type { CalibrationBucket, DrawdownPoint, ForecastBandPoint, PricePoint, RecommendationMarkers } from "./chartTypes";

export const chartFixturePricePoints: PricePoint[] = [
  {
    timestamp: "2026-06-16T10:00:00Z",
    label: "10:00",
    mid: 99800,
    high: 100200,
    low: 99400,
    volume: 262,
    spread: 800,
  },
  {
    timestamp: "2026-06-16T11:00:00Z",
    label: "11:00",
    mid: 99700,
    high: null,
    low: 99700,
    volume: 236,
    spread: null,
  },
  {
    timestamp: "2026-06-16T12:00:00Z",
    label: "12:00",
    mid: 100050,
    high: 100300,
    low: 99800,
    volume: 240,
    spread: 500,
  },
];

export const chartFixtureForecastBand: ForecastBandPoint[] = [
  {
    timestamp: "2026-06-16T10:00:00Z",
    label: "10:00",
    lower: 99450,
    predicted: 99800,
    upper: 100150,
  },
  {
    timestamp: "2026-06-16T11:00:00Z",
    label: "11:00",
    lower: null,
    predicted: 99700,
    upper: null,
  },
  {
    timestamp: "2026-06-16T12:00:00Z",
    label: "12:00",
    lower: 99870,
    predicted: 100050,
    upper: 100260,
  },
];

export const chartFixtureMarkers: RecommendationMarkers = {
  entry: 100000,
  exit: 104000,
  stopLoss: 99000,
};

export const chartFixtureCalibrationBuckets: CalibrationBucket[] = [
  { label: "40-50%", predicted: 0.45, actual: 0.42, sampleSize: 14 },
  { label: "50-60%", predicted: 0.55, actual: 0.53, sampleSize: 19 },
  { label: "60-70%", predicted: 0.65, actual: 0.61, sampleSize: 21 },
];

export const chartFixtureDrawdown: DrawdownPoint[] = [
  { label: "Replay 1", value: 0.06, status: "realized" },
  { label: "Replay 2", value: 0.11, status: "open" },
  { label: "Replay 3", value: null, status: "skipped" },
];
