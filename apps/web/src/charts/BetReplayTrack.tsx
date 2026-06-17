import type { Recommendation, SimulationRun } from "../api/types";

export function BetReplayTrack({
  recommendation,
  simulations,
}: {
  recommendation: Recommendation | null;
  simulations: SimulationRun[];
}) {
  return (
    <div aria-label="Bet replay track" className="timeline-list">
      {simulations.length === 0 ? (
        <span className="timeline-chip">No replay runs yet</span>
      ) : (
        simulations.slice(0, 4).map((run, index) => (
          <span className="timeline-chip" key={run.runId}>
            Replay {index + 1}: {run.name} · {run.status}
          </span>
        ))
      )}
      {recommendation?.strategyVotes[0]?.targetExit !== null && recommendation?.strategyVotes[0]?.targetExit !== undefined ? (
        <span className="timeline-chip">Target exit {recommendation.strategyVotes[0].targetExit} gp</span>
      ) : null}
    </div>
  );
}
