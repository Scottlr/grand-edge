import type { CalibrationBucket } from "./chartTypes";

export function CalibrationBucketGraph({ buckets }: { buckets: CalibrationBucket[] }) {
  return (
    <div aria-label="Confidence honesty graph" className="chart-calibration-grid">
      {buckets.map((bucket) => (
        <div className="chart-calibration-row" key={bucket.label}>
          <strong>{bucket.label}</strong>
          <span>Predicted {Math.round(bucket.predicted * 100)}%</span>
          <span>{bucket.actual === null ? "Actual unavailable" : `Actual ${Math.round(bucket.actual * 100)}%`}</span>
          <span>n={bucket.sampleSize}</span>
        </div>
      ))}
    </div>
  );
}
