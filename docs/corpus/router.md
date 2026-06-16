# Corpus Router

Start every corpus task by picking one route before reading any corpus JSON.
Choose the route whose `helps_with` list best matches the request, then run its
`search_first` commands, then read only the listed docs and source files.

## Route Selection

| Route | Use when the task needs | Read first |
| --- | --- | --- |
| `mechanical_relation_edge` | Item sets, recipes, repairs, alchemy, dose/decant, charge/degrade, substitutes, or category links | `docs/corpus/relation-corpus.md`, `docs/corpus/source-policy.md` |
| `market_event_context` | Updates, events, bot-ban waves, item sinks, activity shifts, or curated event baskets | `docs/corpus/market-intelligence-corpus.md`, `docs/corpus/source-policy.md` |
| `competitor_capability_context` | Product gap analysis only, never user-facing recommendation copy | `docs/corpus/market-intelligence-corpus.md`, `docs/corpus/source-policy.md` |
| `source_policy_review` | Licensing, copied-text risk, review state, source hashes, or summary policy | `docs/corpus/source-policy.md`, `docs/corpus/review-workflow.md` |
| `linked_item_reasoning` | Graph reason atoms, linked-item explanations, and relation-plus-event context | `docs/corpus/relation-corpus.md`, `docs/corpus/market-intelligence-corpus.md` |
| `blast_radius_context` | Event or item shock scenarios, downstream baskets, and contagion reasoning | `docs/corpus/market-intelligence-corpus.md`, `docs/corpus/relation-corpus.md` |
| `ml_graph_feature_context` | Graph feature naming, no-lookahead rules, export contracts, and graph-aware ML context | `docs/corpus/relation-corpus.md`, `docs/corpus/token-budgeting.md` |

## Workflow

1. Match the request to exactly one route.
2. Run the route's `search_first` commands before opening source files.
3. Read only the route's `read_first` docs and at most `max_files_to_open`
   source files.
4. Sample at most `max_sample_entries` entries unless the user explicitly asks
   for a broad audit.
5. Report source ids, review status, content hashes, and graph/corpus versions
   for any corpus-backed conclusion.

## Route Contract

Router JSON entries must follow this shape:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorpusRouteEntry {
    pub route_id: String,
    pub helps_with: Vec<String>,
    pub read_first: Vec<String>,
    pub search_first: Vec<String>,
    pub source_globs: Vec<String>,
    pub max_files_to_open: usize,
    pub max_sample_entries: usize,
    pub stop_and_ask: Vec<String>,
    pub forbidden_full_load: bool,
}
```

Every route must include exact docs, search commands, source globs, sampling
limits, stop conditions, and `forbidden_full_load: true`.

## Stop Conditions

Stop and ask before using corpus material when:

- source licensing is unclear
- an entry is marked `requires_review`
- source ids or content hashes are missing
- a request would expose competitor notes in user-facing copy
- the task appears to require loading full corpus files instead of route-based sampling
