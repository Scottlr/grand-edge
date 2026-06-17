import type { StrategyVoteDto } from "../../domain/strategy";
import { ExpandableAdvancedPanel } from "../disclosure/ExpandableAdvancedPanel";

export function ModelVoteStack({ votes }: { votes: StrategyVoteDto[] }) {
  if (votes.length === 0) {
    return <p className="evidence-empty">No method votes are available yet.</p>;
  }

  return (
    <div className="model-vote-stack">
      {votes.map((vote) => (
        <div className="model-vote-row" key={`${vote.strategyId}-${vote.asOf}`}>
          <div className="model-vote-header">
            <strong>{vote.strategyId}</strong>
            <span>{vote.side.toUpperCase()}</span>
          </div>
          <p>{Math.round(vote.confidence * 100)}% confidence</p>
          <ExpandableAdvancedPanel title="Advanced method detail">
            <dl className="advanced-definition-list">
              <div>
                <dt>Model version</dt>
                <dd>{vote.modelVersion}</dd>
              </div>
              <div>
                <dt>Prediction confidence</dt>
                <dd>{Math.round(vote.confidence * 100)}%</dd>
              </div>
              <div>
                <dt>Trade realism</dt>
                <dd>
                  {vote.execution?.estimatedFillProbability !== null && vote.execution?.estimatedFillProbability !== undefined
                    ? `${Math.round(vote.execution.estimatedFillProbability * 100)}%`
                    : "Unavailable"}
                </dd>
              </div>
            </dl>
          </ExpandableAdvancedPanel>
        </div>
      ))}
    </div>
  );
}
