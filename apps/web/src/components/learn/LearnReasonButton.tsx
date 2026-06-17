import type { GlossaryTermId } from "../../content/glossary";
import { useGlossary } from "./glossaryContext";

export function LearnReasonButton({ termId }: { termId: GlossaryTermId }) {
  const { getEntry, openTerm } = useGlossary();
  const entry = getEntry(termId);

  return (
    <button className="learn-reason-button" onClick={() => openTerm(termId)} type="button">
      Learn: {entry.label}
    </button>
  );
}
