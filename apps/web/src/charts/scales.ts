import type { IntervalPrice } from "../api/types";

export type TimePoint = {
  label: string;
  mid: number | null;
  high: number | null;
  low: number | null;
  spread: number | null;
  volume: number;
};

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

export function valuesExtent(points: TimePoint[]): ChartBounds | null {
  const values = points.flatMap((point) => [point.high, point.low, point.mid]).filter((value): value is number => value !== null);
  if (values.length === 0) {
    return null;
  }

  return { min: Math.min(...values), max: Math.max(...values) };
}

export function volumeExtent(points: TimePoint[]): ChartBounds | null {
  if (points.length === 0) {
    return null;
  }

  const volumes = points.map((point) => point.volume);
  return { min: Math.min(...volumes), max: Math.max(...volumes) };
}
