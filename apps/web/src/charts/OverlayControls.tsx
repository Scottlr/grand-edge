import { useState } from "react";

import { defaultChartLayerLabels, type ChartLayer } from "./chartTypes";

const advancedLayers: ChartLayer[] = ["volume", "spread", "strategyVotes", "regime", "simulationTrades"];

export function OverlayControls({
  layers = advancedLayers,
}: {
  layers?: ChartLayer[];
}) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="chart-overlay-controls">
      <button
        aria-expanded={expanded}
        className="chart-overlay-toggle"
        type="button"
        onClick={() => setExpanded((current) => !current)}
      >
        {expanded ? "Hide advanced chart layers" : "Show advanced chart layers"}
      </button>
      {expanded ? (
        <div className="chart-overlay-list">
          {layers.map((layer) => (
            <span className="timeline-chip" key={layer}>
              {defaultChartLayerLabels[layer]}
            </span>
          ))}
        </div>
      ) : null}
    </div>
  );
}
