import { useRecommendationEvidence } from "../../api/hooks";
import { ModelCardPanel } from "../model-accuracy/ModelCardPanel";
import { EvidenceTimeline } from "./EvidenceTimeline";
import { OutcomeSummaryPanel } from "./OutcomeSummaryPanel";
import { PredictionContributionList } from "./PredictionContributionList";
import { ReasonAtomTable } from "./ReasonAtomTable";
import { ReasonPerformancePanel } from "./ReasonPerformancePanel";

export interface EvidenceTrailViewProps {
  recommendationId: string;
  compact?: boolean;
}

export function EvidenceTrailView({ recommendationId, compact = false }: EvidenceTrailViewProps) {
  const query = useRecommendationEvidence(recommendationId);

  if (query.isLoading) {
    return (
      <article className="terminal-panel">
        <p className="eyebrow">Evidence trail</p>
        <p className="terminal-panel-copy">Loading the stored evidence chain.</p>
      </article>
    );
  }

  if (query.isError) {
    return (
      <article className="terminal-panel">
        <p className="eyebrow">Evidence trail</p>
        <p className="terminal-panel-copy">The evidence trail could not be loaded right now.</p>
      </article>
    );
  }

  const evidence = query.data;
  if (!evidence) {
    return (
      <article className="terminal-panel">
        <p className="eyebrow">Evidence trail</p>
        <p className="terminal-panel-copy">No evidence was returned for this recommendation.</p>
      </article>
    );
  }

  return (
    <section className="detailed-view-stack">
      <article className="terminal-panel">
        <p className="eyebrow">Evidence trail</p>
        <p className="terminal-panel-copy">
          {evidence.dataState.reason ?? "This view reconstructs what the system saw, predicted, recommended, and learned later."}
        </p>
        <p className="terminal-panel-copy">{evidence.explanation.summary}</p>
      </article>
      <EvidenceTimeline stages={evidence.stages} />
      <PredictionContributionList predictionLinks={evidence.predictionLinks} predictions={evidence.predictions} />
      <ReasonAtomTable reasons={evidence.explanation.reasonAtoms} />
      {!compact ? <ReasonPerformancePanel rows={evidence.reasonPerformance} /> : null}
      <OutcomeSummaryPanel outcome={evidence.outcome} pending={evidence.dataState.status === "pending"} />
      {!compact ? <ModelCardPanel modelCards={evidence.modelCards} /> : null}
      {!compact && evidence.graphVersion ? (
        <article className="terminal-panel">
          <p className="eyebrow">Graph context</p>
          <div className="terminal-list">
            {evidence.graphPaths.map((path, index) => (
              <div className="terminal-list-row" key={`${path.edgeId ?? "graph"}-${index}`}>
                <div>
                  <strong>{path.relationType.replaceAll("_", " ")}</strong>
                  <p>
                    {path.sourceItemId} to {path.targetItemId}
                  </p>
                </div>
                <div>
                  <strong>{path.contributionWeight === null ? "Unavailable" : `${Math.round(path.contributionWeight * 100)}%`}</strong>
                  <p>{evidence.graphVersion}</p>
                </div>
              </div>
            ))}
          </div>
        </article>
      ) : null}
    </section>
  );
}
