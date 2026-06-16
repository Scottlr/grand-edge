# Relation Corpus

The planned relation corpus under `data/relations/` holds high-confidence,
versioned mechanical relationships that later import into graph edges.

Expected files include:

- `source_registry.v1.json`
- `item_sets.v1.json`
- `recipes.v1.json`
- `repairs.v1.json`
- `alchemy.v1.json`
- `dose_decant.v1.json`
- `charge_links.v1.json`
- `degrade_links.v1.json`
- `categories.v1.json`
- `substitutes.v1.json`
- `market_analysis_sources.v1.json`

Use this corpus for trusted mechanical links first. If a relationship is only a
name pattern or a hypothesis, it belongs in a low-confidence reviewed path, not
as a trusted edge.
