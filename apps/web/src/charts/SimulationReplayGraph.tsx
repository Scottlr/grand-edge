import type { PricePoint, RecommendationMarkers } from "./chartTypes";
import { PricePathGraph } from "./PricePathGraph";

export function SimulationReplayGraph({
  points,
  markers,
  replayLabels,
}: {
  points: PricePoint[];
  markers?: RecommendationMarkers | null;
  replayLabels: string[];
}) {
  return (
    <div className="chart-stack">
      <PricePathGraph markers={markers} points={points} />
      <div aria-label="Past test trades" className="timeline-list">
        {replayLabels.length === 0 ? (
          <span className="timeline-chip">No past test trades yet</span>
        ) : (
          replayLabels.map((label) => (
            <span className="timeline-chip" key={label}>
              {label}
            </span>
          ))
        )}
      </div>
    </div>
  );
}
