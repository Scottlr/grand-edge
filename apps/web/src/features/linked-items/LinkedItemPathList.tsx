import type { LinkedItemPath } from "./linkedItemTypes";

function percent(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }

  return `${Math.round(value * 100)}%`;
}

export function LinkedItemPathList({ paths }: { paths: LinkedItemPath[] }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Linked item paths</p>
      <div className="terminal-list">
        {paths.map((path, index) => (
          <div
            className="terminal-list-row linked-path-row"
            key={`${path.label}-${path.sourceItemId}-${index}`}
          >
            <div>
              <strong>{path.label}</strong>
              <p>
                {path.sourceItemId} to {path.targetItemName}
              </p>
              <p className="linked-path-caveat">
                {path.sourceType === "learned"
                  ? "Predictive evidence only. This does not claim one item causes the other."
                  : `${path.sourceType} source evidence.`}
              </p>
            </div>
            <div className="linked-path-stats">
              <strong>{percent(path.pathConfidence)}</strong>
              <p>{path.graphVersion ?? "Graph version unavailable"}</p>
            </div>
          </div>
        ))}
      </div>
    </article>
  );
}
