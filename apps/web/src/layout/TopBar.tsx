import { Activity, Bell, Database, RefreshCcw, Wifi, WifiOff } from "lucide-react";

import type { LiveConnectionState } from "../state/workspaceStore";
import { TooltipTerm } from "../components/learn/TooltipTerm";

type TopBarProps = {
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

export function TopBar({
  liveConnectionState,
  recommendationCount,
  simulationCount,
  strategyCount,
}: TopBarProps) {
  return (
    <header className="terminal-topbar">
      <div className="terminal-topbar-copy">
        <p className="eyebrow">Grand Edge terminal</p>
        <h1>Command center</h1>
        <p className="terminal-topbar-detail">
          Scan backend-owned recommendations, method availability, positions, and{" "}
          <TooltipTerm term="simulation">simulations</TooltipTerm> from one operational surface. Richer
          trust details and deeper analytical views land in later slices.
        </p>
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
