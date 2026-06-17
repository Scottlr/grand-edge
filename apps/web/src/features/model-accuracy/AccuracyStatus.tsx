import { accuracyStatusForModel, type ModelAccuracyViewModel } from "../../domain/modelAccuracy";

const labels = {
  well_calibrated: "Confidence honesty looks healthy.",
  under_confident: "The model has been more cautious than the outcomes suggest.",
  over_confident: "The model has sounded too certain for the outcomes it delivered.",
  insufficient_sample: "Not enough recent examples yet.",
  unavailable: "Accuracy detail is unavailable right now.",
} as const;

export function AccuracyStatus({ model }: { model: ModelAccuracyViewModel }) {
  const status = accuracyStatusForModel(model);

  return (
    <article className="terminal-panel">
      <p className="eyebrow">Accuracy status</p>
      <strong>{labels[status]}</strong>
      <p className="terminal-panel-copy">Sample size: {model.sampleSize}</p>
    </article>
  );
}
