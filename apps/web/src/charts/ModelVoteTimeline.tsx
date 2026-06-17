import type { StrategyVoteDto } from "../domain/strategy";

export function ModelVoteTimeline({ votes }: { votes: StrategyVoteDto[] }) {
  return (
    <div aria-label="Advanced method views timeline" className="chart-timeline-grid">
      {votes.map((vote) => (
        <div className="chart-timeline-row" key={`${vote.strategyId}-${vote.modelVersion}`}>
          <strong>{vote.strategyId}</strong>
          <span>{Math.round(vote.confidence * 100)}%</span>
          <span>{vote.side}</span>
        </div>
      ))}
    </div>
  );
}
