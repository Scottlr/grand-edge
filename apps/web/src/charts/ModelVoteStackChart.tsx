import type { StrategyVoteDto } from "../domain/strategy";

export function ModelVoteStackChart({ votes }: { votes: StrategyVoteDto[] }) {
  return (
    <div aria-label="Model vote stack" className="stacked-votes">
      {votes.map((vote) => (
        <div className="stacked-vote-row" key={`${vote.strategyId}-${vote.modelVersion}`}>
          <strong>{vote.strategyId}</strong>
          <span>{Math.round(vote.confidence * 100)}%</span>
        </div>
      ))}
    </div>
  );
}
