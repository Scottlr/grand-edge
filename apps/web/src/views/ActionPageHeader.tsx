import type { ReactNode } from "react";

import { TooltipTerm } from "../components/learn/TooltipTerm";
import type { SimpleActionLabel } from "../components/recommendation/recommendationFixtures";

export type ActionPageHeaderProps = {
  action: SimpleActionLabel;
  itemName?: string;
  why: string;
  confidence: number | null;
  keyNumbers: ReactNode;
  actions: ReactNode;
};

export function ActionPageHeader({
  action,
  actions,
  confidence,
  itemName,
  keyNumbers,
  why,
}: ActionPageHeaderProps) {
  return (
    <article className="terminal-panel action-page-header">
      <div className="action-page-heading">
        <div>
          <p className="eyebrow">Suggested action</p>
          <h2>
            {action}
            {itemName ? ` ${itemName}` : ""}
          </h2>
        </div>
        <div className="action-page-confidence">
          <span className="terminal-mono">
            <TooltipTerm term="confidence">Confidence</TooltipTerm>
          </span>
          <strong>{confidence === null ? "Unavailable" : `${Math.round(confidence * 100)}%`}</strong>
        </div>
      </div>
      <p className="terminal-panel-copy">{why}</p>
      <div className="action-page-keynumbers">{keyNumbers}</div>
      <div className="action-page-actions">{actions}</div>
    </article>
  );
}
