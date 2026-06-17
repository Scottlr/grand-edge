import type { BlastRadiusImpact } from "./linkedItemTypes";

function percent(value: number) {
  return `${Math.round(value * 100)}%`;
}

export function WhatIfThisMovesTree({
  impacts,
}: {
  impacts: BlastRadiusImpact[];
}) {
  const groups = [
    { title: "First-order impacts", rows: impacts.slice(0, 1) },
    { title: "Second-order impacts", rows: impacts.slice(1, 2) },
    { title: "Third-order impacts", rows: impacts.slice(2, 3) },
  ];

  return (
    <div className="linked-impact-groups">
      {groups.map((group) => (
        <article className="terminal-panel" key={group.title}>
          <p className="eyebrow">{group.title}</p>
          {group.rows.length === 0 ? (
            <p className="terminal-panel-copy">No stored impacts at this depth.</p>
          ) : (
            <div className="terminal-list">
              {group.rows.map((impact) => (
                <div
                  className="terminal-list-row"
                  key={`${group.title}-${impact.itemId}`}
                >
                  <div>
                    <strong>{impact.itemName}</strong>
                    <p>{impact.path.label}</p>
                  </div>
                  <div>
                    <strong>{percent(impact.confidence)}</strong>
                    <p>
                      Expected impact {impact.expectedImpact > 0 ? "+" : ""}
                      {percent(impact.expectedImpact)}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </article>
      ))}
    </div>
  );
}
