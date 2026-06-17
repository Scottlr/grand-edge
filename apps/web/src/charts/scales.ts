import type { IntervalPrice } from "../api/types";
import type { ForecastBandPoint, PricePoint, RecommendationMarkers, TimePoint } from "./chartTypes";

export type ChartBounds = {
  min: number;
  max: number;
};

function midpoint(high: number | null, low: number | null): number | null {
  if (high === null && low === null) {
    return null;
  }
  if (high === null) {
    return low;
  }
  if (low === null) {
    return high;
  }

  return Math.round((high + low) / 2);
}

export function intervalPricesToTimePoints(prices: IntervalPrice[]): TimePoint[] {
  return prices.map((price) => {
    const high = price.avgHighPrice;
    const low = price.avgLowPrice;

    return {
      timestamp: price.bucketStart,
      label: new Date(price.bucketStart).toLocaleTimeString("en-GB", {
        hour: "2-digit",
        minute: "2-digit",
        timeZone: "UTC",
      }),
      mid: midpoint(high, low),
      high,
      low,
      spread: high !== null && low !== null ? high - low : null,
      volume: price.highPriceVolume + price.lowPriceVolume,
    };
  });
}

export function timePointsToPricePoints(points: TimePoint[]): PricePoint[] {
  return points.map((point) => ({
    timestamp: point.timestamp,
    label: point.label,
    mid: point.mid,
    high: point.high,
    low: point.low,
    volume: point.volume,
    spread: point.spread,
  }));
}

export function buildForecastBand(points: PricePoint[]): ForecastBandPoint[] {
  return points.map((point) => ({
    timestamp: point.timestamp,
    label: point.label,
    lower: point.mid === null ? null : point.mid - 180,
    predicted: point.mid,
    upper: point.mid === null ? null : point.mid + 220,
  }));
}

export function recommendationMarkersFromVote(vote: {
  targetEntry: number | null;
  targetExit: number | null;
  stopLoss: number | null;
} | null | undefined): RecommendationMarkers {
  return {
    entry: vote?.targetEntry ?? null,
    exit: vote?.targetExit ?? null,
    stopLoss: vote?.stopLoss ?? null,
  };
}

export function valuesExtent(points: TimePoint[]): ChartBounds | null {
  const values = points.flatMap((point) => [point.high, point.low, point.mid]).filter((value): value is number => value !== null);
  if (values.length === 0) {
    return null;
  }

  return { min: Math.min(...values), max: Math.max(...values) };
}

export function volumeExtent(points: Array<{ volume: number | null }>): ChartBounds | null {
  if (points.length === 0) {
    return null;
  }

  const volumes = points.map((point) => point.volume ?? 0);
  return { min: Math.min(...volumes), max: Math.max(...volumes) };
}
