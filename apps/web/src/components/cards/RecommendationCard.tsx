import type { GlossaryTermId } from "../../content/glossary";
import type { DataState, RiskLabel } from "../../domain/recommendation";
import { ConfidenceMeter } from "../confidence/ConfidenceMeter";
import { EvidenceStack } from "../evidence/EvidenceStack";
import { InvalidationRules } from "../evidence/InvalidationRules";
import { ExpandableAdvancedPanel } from "../disclosure/ExpandableAdvancedPanel";
import { ModelVoteStack } from "../strategy/ModelVoteStack";
import type { StrategyVoteDto } from "../../domain/strategy";

export type RecommendationCardProps = {
  itemName: string;
  action: "BUY" | "SELL" | "WAIT" | "HOLD" | "SELL ALL" | "SELL SOME" | "DO NOT ADD" | "WATCH CLOSELY";
  confidence: number;
  expectedNetGp: number | null;
  expectedRoi: number | null;
  horizonLabel: string;
  riskLabel: RiskLabel;
  primaryReason: string;
  reasons: string[];
  learnTermIds: GlossaryTermId[];
  modelAgreement: number;
  dataState: DataState;
  confidenceBreakdown: {
    confidence: number;
    modelAgreementLabel: string;
    recentAccuracy: number | null;
    dataQualityLabel: string;
  };
  strategyVotes: StrategyVoteDto[];
  invalidationRules: Array<{
    metric: string;
    operator: string;
    threshold: string;
    currentValue: string | null;
    reason: string;
  }>;
};

function formatValue(value: number | null, kind: "gp" | "roi"): string {
  if (value === null) {
    return "Unavailable";
  }

  if (kind === "roi") {
    return `${Math.round(value * 100)}%`;
  }

  return `${new Intl.NumberFormat("en-GB").format(value)} gp`;
}

export function RecommendationCard({
  action,
  confidence,
  confidenceBreakdown,
  dataState,
  expectedNetGp,
  expectedRoi,
  horizonLabel,
  invalidationRules,
  itemName,
  learnTermIds,
  modelAgreement,
  primaryReason,
  reasons,
  riskLabel,
  strategyVotes,
}: RecommendationCardProps) {
  return (
    <article className="recommendation-card">
      <div className="recommendation-card-header">
        <div>
          <p className="eyebrow">Suggested action</p>
          <h3>{itemName}</h3>
        </div>
        <span className="recommendation-card-action">{action}</span>
      </div>

      <div className="recommendation-card-metrics">
        <div>
          <span className="eyebrow">Expected profit</span>
          <strong>{formatValue(expectedNetGp, "gp")}</strong>
        </div>
        <div>
          <span className="eyebrow">Return</span>
          <strong>{formatValue(expectedRoi, "roi")}</strong>
        </div>
        <div>
          <span className="eyebrow">Window</span>
          <strong>{horizonLabel}</strong>
        </div>
        <div>
          <span className="eyebrow">Risk</span>
          <strong>{riskLabel}</strong>
        </div>
      </div>

      <EvidenceStack learnTermIds={learnTermIds} primaryReason={primaryReason} reasons={reasons} />
      <ConfidenceMeter
        confidence={confidence}
        dataQualityLabel={confidenceBreakdown.dataQualityLabel}
        modelAgreementLabel={confidenceBreakdown.modelAgreementLabel}
        recentAccuracy={confidenceBreakdown.recentAccuracy}
      />

      <ExpandableAdvancedPanel title="Advanced recommendation detail">
        <dl className="advanced-definition-list">
          <div>
            <dt>Recommendation confidence</dt>
            <dd>{Math.round(confidenceBreakdown.confidence * 100)}%</dd>
          </div>
          <div>
            <dt>Data state</dt>
            <dd>{dataState}</dd>
          </div>
          <div>
            <dt>Model agreement</dt>
            <dd>{Math.round(modelAgreement * 100)}%</dd>
          </div>
        </dl>
        <InvalidationRules rules={invalidationRules} />
        <ModelVoteStack votes={strategyVotes} />
      </ExpandableAdvancedPanel>
    </article>
  );
}
