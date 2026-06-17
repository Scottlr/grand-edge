import type { Recommendation } from "../api/types";

export function RegimeTimeline({ recommendation }: { recommendation: Recommendation | null }) {
  const labels = [
    recommendation?.confidenceBreakdown.dataQualityLabel ?? "unknown data",
    recommendation?.confidenceBreakdown.executionQualityLabel ?? "unknown fills",
    recommendation?.confidenceBreakdown.modelAgreementLabel ?? "unknown agreement",
  ];

  return (
    <div aria-label="Market mood timeline" className="timeline-list">
      {labels.map((label) => (
        <span className="timeline-chip" key={label}>
          {label}
        </span>
      ))}
    </div>
  );
}
