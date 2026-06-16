---
name: grandedge-corpus-router
description: Use when Codex needs to inspect, amend, validate, or reason from the GrandEdge relation or market intelligence corpus without loading large corpus files. Routes agents to the smallest relevant docs, schemas, search commands, and source files for tasks involving item relationships, market events, source policy, corpus review, graph evidence, linked items, or blast-radius context.
---

# GrandEdge Corpus Router

Open `docs/corpus/router.md` first. Pick exactly one route before reading any
corpus data.

## Workflow

1. Match the request to the route whose `helps_with` list fits best.
2. Run the route's `search_first` commands before opening source files.
3. Read only the route's `read_first` docs and at most `max_files_to_open`
   source files.
4. Sample at most `max_sample_entries` entries unless the user explicitly asks
   for a broad corpus audit.
5. Report route id, source ids, review status, content hashes, and graph/corpus
   version context for any corpus-backed conclusion.

## Guardrails

- Never paste full corpus files or large JSON arrays into chat.
- Treat corpus notes as sourced context and hypotheses, not market truth.
- Do not use competitor capability notes in user-facing recommendation copy.
- Stop if source licensing, review state, or end-user visibility is unclear.
- Stop if the task appears to require broad full-corpus loading instead of route
  sampling.

## References

- `references/router.md`: thin start-here pointer for the route-first workflow.
- `references/corpus-map.md`: route ids and when to use them.
