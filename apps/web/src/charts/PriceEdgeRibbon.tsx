import { PricePathGraph } from "./PricePathGraph";
import { recommendationMarkersFromVote } from "./scales";
import type { ForecastBandPoint, PricePoint } from "./chartTypes";

export function PriceEdgeRibbon({
  points,
  markers,
  forecastBand,
}: {
  points: PricePoint[];
  markers?: {
    targetEntry: number | null;
    targetExit: number | null;
    stopLoss: number | null;
  } | null;
  forecastBand?: ForecastBandPoint[] | null;
}) {
  return <PricePathGraph forecastBand={forecastBand ?? null} markers={recommendationMarkersFromVote(markers)} points={points} />;
}
