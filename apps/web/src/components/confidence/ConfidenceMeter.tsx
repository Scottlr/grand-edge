import { TooltipTerm } from "../learn/TooltipTerm";
import { getConfidenceState, toConfidencePercent } from "./confidenceState";

export type ConfidenceMeterProps = {
  confidence: number;
  modelAgreementLabel: string;
  recentAccuracy: number | null;
  dataQualityLabel: string;
};

export function ConfidenceMeter({
  confidence,
  modelAgreementLabel,
  recentAccuracy,
  dataQualityLabel,
}: ConfidenceMeterProps) {
  const normalized = toConfidencePercent(confidence);
  const state = getConfidenceState(confidence);

  return (
    <section className={`confidence-meter confidence-meter-${state}`}>
      <div className="confidence-meter-header">
        <div>
          <p className="eyebrow">Confidence</p>
          <strong>{normalized}%</strong>
        </div>
        <span className={`confidence-state confidence-state-${state}`}>{state}</span>
      </div>
      <div className="confidence-meter-bar" aria-hidden="true">
        <span className="confidence-meter-fill" style={{ width: `${normalized}%` }} />
      </div>
      <dl className="confidence-meter-details">
        <div>
          <dt>
            <TooltipTerm term="confidence">Confidence</TooltipTerm>
          </dt>
          <dd>{modelAgreementLabel}</dd>
        </div>
        <div>
          <dt>
            <TooltipTerm term="modelAccuracy">Past accuracy</TooltipTerm>
          </dt>
          <dd>{recentAccuracy === null ? "Not enough history yet." : `${Math.round(recentAccuracy * 100)}% recently`}</dd>
        </div>
        <div>
          <dt>
            <TooltipTerm term="dataQuality">Data quality</TooltipTerm>
          </dt>
          <dd>{dataQualityLabel}</dd>
        </div>
      </dl>
    </section>
  );
}
