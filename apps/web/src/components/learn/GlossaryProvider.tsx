import { useMemo, useState } from "react";
import type { ReactNode } from "react";

import { getGlossaryEntry, type GlossaryTermId } from "../../content/glossary";
import { LearnModal } from "./LearnModal";
import { GlossaryContext, type GlossaryContextValue } from "./glossaryContext";

export function GlossaryProvider({ children }: { children: ReactNode }) {
  const [activeTerm, setActiveTerm] = useState<GlossaryTermId | null>(null);

  const value = useMemo<GlossaryContextValue>(
    () => ({
      getEntry: getGlossaryEntry,
      openTerm: setActiveTerm,
      closeTerm: () => setActiveTerm(null),
      activeTerm,
    }),
    [activeTerm],
  );

  return (
    <GlossaryContext.Provider value={value}>
      {children}
      {activeTerm ? <LearnModal term={activeTerm} open onOpenChange={(open) => !open && setActiveTerm(null)} /> : null}
    </GlossaryContext.Provider>
  );
}
