import { createContext, useContext } from "react";

import type { GlossaryEntry, GlossaryTermId } from "../../content/glossary";

export type GlossaryContextValue = {
  getEntry: (term: GlossaryTermId) => GlossaryEntry;
  openTerm: (term: GlossaryTermId) => void;
  closeTerm: () => void;
  activeTerm: GlossaryTermId | null;
};

export const GlossaryContext = createContext<GlossaryContextValue | null>(null);

export function useGlossary() {
  const context = useContext(GlossaryContext);

  if (!context) {
    throw new Error("useGlossary must be used within a GlossaryProvider");
  }

  return context;
}
