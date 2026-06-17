import { useId, useState, type ReactNode } from "react";

import type { GlossaryTermId } from "../../content/glossary";
import { useGlossary } from "./glossaryContext";

export type TooltipTermProps = {
  term: GlossaryTermId;
  children?: ReactNode;
};

export function TooltipTerm({ term, children }: TooltipTermProps) {
  const [open, setOpen] = useState(false);
  const tooltipId = useId();
  const { getEntry, openTerm } = useGlossary();
  const entry = getEntry(term);

  return (
    <span className="learn-term">
      <button
        aria-controls={tooltipId}
        aria-expanded={open}
        aria-haspopup="dialog"
        className="learn-term-button"
        onBlur={() => setOpen(false)}
        onClick={() => setOpen((current) => !current)}
        onFocus={() => setOpen(true)}
        onMouseEnter={() => setOpen(true)}
        onMouseLeave={() => setOpen(false)}
        type="button"
      >
        {children ?? entry.label}
      </button>

      {open ? (
        <span className="learn-tooltip" id={tooltipId} role="tooltip">
          <strong>{entry.label}</strong>
          <span>{entry.quick}</span>
          <button className="learn-inline-link" onClick={() => openTerm(term)} type="button">
            Learn more
          </button>
        </span>
      ) : null}
    </span>
  );
}
