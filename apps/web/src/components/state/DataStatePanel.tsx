import type { ReactNode } from "react";

import type { DataState } from "../../domain/recommendation";

export type DataStatePanelProps = {
  state: DataState;
  title?: string;
  message?: string;
  action?: ReactNode;
};

const defaultContent: Record<DataState, { title: string; message: string }> = {
  loading: {
    title: "Loading this view",
    message: "GrandEdge is fetching the latest backend data for this panel.",
  },
  live: {
    title: "Live data",
    message: "Backend data is available for this view.",
  },
  stale: {
    title: "Data is stale",
    message: "Data is stale. Recommendations are paused until fresh prices arrive.",
  },
  degraded: {
    title: "Data is degraded",
    message: "GrandEdge is showing the safe fallback instead of inventing missing pieces.",
  },
  empty: {
    title: "Nothing to show yet",
    message: "This view does not have enough stored data to show a result yet.",
  },
  error: {
    title: "This view hit an error",
    message: "GrandEdge is showing the error honestly instead of inventing advice.",
  },
};

export function DataStatePanel({
  state,
  title,
  message,
  action,
}: DataStatePanelProps) {
  const content = defaultContent[state];

  return (
    <article
      aria-live={state === "loading" ? "polite" : "assertive"}
      className={`terminal-panel data-state-panel data-state-panel-${state}`}
      role="status"
    >
      <p className="eyebrow">Data state</p>
      <h3>{title ?? content.title}</h3>
      <p className="terminal-panel-copy">{message ?? content.message}</p>
      {action ? <div className="data-state-panel-action">{action}</div> : null}
    </article>
  );
}
