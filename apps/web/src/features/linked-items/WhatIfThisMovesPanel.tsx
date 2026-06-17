import { useState } from "react";

import { TooltipTerm } from "../../components/learn/TooltipTerm";
import { WhatIfThisMovesTree } from "./WhatIfThisMovesTree";
import type { BlastRadiusImpact } from "./linkedItemTypes";

export function WhatIfThisMovesPanel({
  impacts,
  graphVersion,
}: {
  impacts: BlastRadiusImpact[];
  graphVersion: string | null;
}) {
  const [shockSize, setShockSize] = useState(4);
  const [depth, setDepth] = useState(3);
  const [threshold, setThreshold] = useState(40);
  const [mode, setMode] = useState<"watch" | "stress" | "best_case">("watch");

  const filtered = impacts.filter((impact, index) => {
    const depthAllowed = index < depth;
    const confidenceAllowed = impact.confidence * 100 >= threshold;
    return depthAllowed && confidenceAllowed;
  });

  return (
    <section className="linked-items-stack">
      <article className="terminal-panel">
        <p className="eyebrow">
          <TooltipTerm term="blastRadius">
            What happens if this moves?
          </TooltipTerm>
        </p>
        <h3>What happens if this moves?</h3>
        <p className="terminal-panel-copy">
          Advanced subtitle: blast radius simulation. The default wording stays
          user-facing.
        </p>
        <div className="linked-form-grid">
          <label>
            <span>Shock size</span>
            <input
              type="range"
              min="1"
              max="10"
              value={shockSize}
              onChange={(event) => setShockSize(Number(event.target.value))}
            />
            <strong>{shockSize}%</strong>
          </label>
          <label>
            <span>Scenario mode</span>
            <select
              value={mode}
              onChange={(event) =>
                setMode(
                  event.target.value as "watch" | "stress" | "best_case",
                )
              }
            >
              <option value="watch">Watch-first</option>
              <option value="stress">Stress test</option>
              <option value="best_case">Best-case</option>
            </select>
          </label>
          <label>
            <span>Depth</span>
            <input
              type="number"
              min="1"
              max="3"
              value={depth}
              onChange={(event) => setDepth(Number(event.target.value) || 1)}
            />
          </label>
          <label>
            <span>Confidence threshold</span>
            <input
              type="number"
              min="0"
              max="100"
              value={threshold}
              onChange={(event) => setThreshold(Number(event.target.value) || 0)}
            />
          </label>
        </div>
        <p className="terminal-panel-copy">
          {graphVersion
            ? `Using graph version ${graphVersion}. Confidence naturally decays as path depth increases.`
            : "Graph version is unavailable, so these what-if examples are shown as fixture guidance only."}
        </p>
      </article>
      <WhatIfThisMovesTree impacts={filtered} />
    </section>
  );
}
