import type { GlossaryTermId } from "../../content/glossary";
import { useGlossary } from "./glossaryContext";

export type LearnModalProps = {
  term: GlossaryTermId;
  open: boolean;
  onOpenChange(open: boolean): void;
};

export function LearnModal({ term, open, onOpenChange }: LearnModalProps) {
  const { getEntry } = useGlossary();
  const entry = getEntry(term);

  if (!open) {
    return null;
  }

  return (
    <div className="learn-modal-backdrop" role="presentation" onClick={() => onOpenChange(false)}>
      <section
        aria-label={`${entry.learnTitle} learn panel`}
        aria-modal="true"
        className="learn-modal"
        role="dialog"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="learn-modal-header">
          <div>
            <p className="eyebrow">Learn</p>
            <h2>{entry.learnTitle}</h2>
          </div>
          <button className="terminal-icon-button" onClick={() => onOpenChange(false)} type="button">
            Close
          </button>
        </div>

        <p className="learn-modal-quick">{entry.quick}</p>
        <div className="learn-copy-stack">
          {entry.learnBody.map((paragraph) => (
            <p key={paragraph}>{paragraph}</p>
          ))}
        </div>

        {entry.example ? (
          <div className="learn-detail-block">
            <p className="eyebrow">Example</p>
            <p>{entry.example}</p>
          </div>
        ) : null}

        {entry.whyItMatters?.length ? (
          <div className="learn-detail-block">
            <p className="eyebrow">Why it matters</p>
            <ul className="learn-list">
              {entry.whyItMatters.map((line) => (
                <li key={line}>{line}</li>
              ))}
            </ul>
          </div>
        ) : null}

        {entry.advanced?.length ? <AdvancedTermPanel term={term} /> : null}
      </section>
    </div>
  );
}

export type AdvancedTermPanelProps = {
  term: GlossaryTermId;
};

export function AdvancedTermPanel({ term }: AdvancedTermPanelProps) {
  const { getEntry } = useGlossary();
  const entry = getEntry(term);

  if (!entry.advanced?.length) {
    return null;
  }

  return (
    <div className="learn-detail-block learn-detail-block-advanced">
      <p className="eyebrow">Advanced</p>
      <ul className="learn-list">
        {entry.advanced.map((line) => (
          <li key={line}>{line}</li>
        ))}
      </ul>
    </div>
  );
}
