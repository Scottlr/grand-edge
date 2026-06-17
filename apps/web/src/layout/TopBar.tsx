import { Activity, Bell, Database, RefreshCcw, Wifi, WifiOff } from "lucide-react";

import { primaryNavItems } from "../navigation/routes";
import type { LiveConnectionState } from "../state/workspaceStore";
import type { WorkspaceView } from "../state/workspaceStore";

type TopBarProps = {
  activeView: WorkspaceView;
  liveConnectionState: LiveConnectionState;
  recommendationCount: number;
  simulationCount: number;
  strategyCount: number;
};

function liveStateLabel(state: LiveConnectionState) {
  switch (state) {
    case "live":
      return "Live stream connected";
    case "connecting":
      return "Connecting stream";
    case "error":
      return "Live stream degraded";
    case "closed":
      return "Live stream paused";
    case "idle":
    default:
      return "Live stream idle";
  }
}

function viewHeading(view: WorkspaceView) {
  return primaryNavItems.find((item) => item.id === view)?.label ?? "Dashboard";
}

function viewDescription(view: WorkspaceView) {
  switch (view) {
    case "buy":
      return "Review the clearest buy case with confidence, expected upside, and invalidation rules before acting.";
    case "sell":
      return "Review the clearest exit case with profit, confidence, and trade realism before selling.";
    case "portfolio":
      return "Check current holdings, update tracked positions, and compare live exit guidance from one place.";
    case "items":
      return "Open item intelligence for price shape, spread, liquidity, model votes, and recent accuracy in one view.";
    case "linkedItems":
      return "Follow linked-item context first, then dive deeper only when the relationship changes the action.";
    case "simulations":
      return "Compare replay outcomes against the observed market path before trusting a setup again.";
    case "accuracy":
      return "Start with trust signals, recent accuracy, and confidence honesty before model jargon.";
    case "settings":
      return "Keep advanced controls secondary so the main workflow stays centered on clear actions.";
    case "dashboard":
    default:
      return "Scan backend-owned recommendations, method availability, positions, and simulations from one operational surface. Richer trust details and deeper analytical views land in later slices.";
  }
}

export function TopBar({
  activeView,
  liveConnectionState,
  recommendationCount,
  simulationCount,
  strategyCount,
}: TopBarProps) {
  return (
    <header className="terminal-topbar">
      <div className="terminal-topbar-copy">
        <p className="eyebrow">Grand Edge terminal</p>
        <h1>{viewHeading(activeView)}</h1>
        <p className="terminal-topbar-detail">{viewDescription(activeView)}</p>
      </div>

      <div className="terminal-status-cluster" aria-label="terminal status">
        <span className="terminal-status-pill">
          {liveConnectionState === "live" ? <Wifi size={16} /> : <WifiOff size={16} />}
          {liveStateLabel(liveConnectionState)}
        </span>
        <span className="terminal-status-pill">
          <Bell size={16} />
          {recommendationCount} recommendations
        </span>
        <span className="terminal-status-pill">
          <Activity size={16} />
          {simulationCount} simulations
        </span>
        <span className="terminal-status-pill">
          <RefreshCcw size={16} />
          {strategyCount} strategies
        </span>
        <span className="terminal-status-pill">
          <Database size={16} />
          API-backed shell
        </span>
      </div>
    </header>
  );
}
