import type { PredictionEvidence, PredictionLink } from "../../domain/evidence";

function formatPercent(value: number | null) {
  if (value === null) {
    return "Unavailable";
  }
  return `${Math.round(value * 100)}%`;
}

export function PredictionContributionList({
  predictions,
  predictionLinks,
}: {
  predictions: PredictionEvidence[];
  predictionLinks: PredictionLink[];
}) {
  const linksByPredictionId = new Map(predictionLinks.map((link) => [link.predictionId, link]));

  return (
    <article className="terminal-panel">
      <p className="eyebrow">Prediction stage</p>
      {predictions.length === 0 ? (
        <p className="terminal-empty-state">No linked predictions were stored for this recommendation.</p>
      ) : (
        <div className="terminal-list">
          {predictions.map((prediction) => {
            const link = linksByPredictionId.get(prediction.predictionId);
            return (
              <div className="terminal-list-row" key={prediction.predictionId}>
                <div>
                  <strong>{prediction.modelId}</strong>
                  <p>{prediction.modelVersion}</p>
                </div>
                <div>
                  <strong>{prediction.predictedDirection}</strong>
                  <p>
                    Contribution {link ? formatPercent(link.contributionWeight) : "Unavailable"} · Confidence{" "}
                    {formatPercent(prediction.confidence)}
                  </p>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </article>
  );
}
