import { AlertTriangle, CheckCircle2, Sparkles } from "lucide-react";

import type { GlossaryTermId } from "../../content/glossary";
import { LearnReasonButton } from "../learn/LearnReasonButton";

export function EvidenceStack({
  primaryReason,
  reasons,
  learnTermIds,
}: {
  primaryReason: string;
  reasons: string[];
  learnTermIds: GlossaryTermId[];
}) {
  return (
    <section className="evidence-stack">
      <div className="evidence-primary">
        <Sparkles size={16} />
        <p>{primaryReason}</p>
      </div>
      <details className="evidence-reasons">
        <summary>Show why</summary>
        {reasons.length > 0 ? (
          <ul className="evidence-reason-list">
            {reasons.map((reason, index) => (
              <li key={`${reason}-${index}`}>
                {reason.toLowerCase().includes("weak") || reason.toLowerCase().includes("uncertain") ? (
                  <AlertTriangle size={14} />
                ) : (
                  <CheckCircle2 size={14} />
                )}
                <span>{reason}</span>
              </li>
            ))}
          </ul>
        ) : (
          <p className="evidence-empty">No extra reasons are available yet.</p>
        )}
      </details>
      <div className="learn-reason-list">
        {learnTermIds.map((termId) => (
          <LearnReasonButton key={termId} termId={termId} />
        ))}
      </div>
    </section>
  );
}
