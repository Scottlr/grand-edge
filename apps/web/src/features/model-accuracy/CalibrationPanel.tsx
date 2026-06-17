import type { ModelAccuracyViewModel } from "../../domain/modelAccuracy";
import { CalibrationBucketGraph } from "../../charts/CalibrationBucketGraph";

export function CalibrationPanel({ model }: { model: ModelAccuracyViewModel }) {
  return (
    <article className="terminal-panel">
      <p className="eyebrow">Confidence honesty</p>
      <p className="terminal-panel-copy">Recent sample size: {model.sampleSize}</p>
      <CalibrationBucketGraph buckets={model.calibrationBuckets} />
      <p className="terminal-panel-copy">
        {model.calibrationBuckets.length > 0
          ? "When GrandEdge sounded more certain, this view checks whether the outcomes really kept up."
          : "Confidence honesty is unavailable right now."}
      </p>
    </article>
  );
}
