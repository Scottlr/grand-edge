# Review Workflow

1. Create or update the source registry entry with `source_id`, license note,
   retrieval time, and `content_hash`.
2. Add or update the smallest relevant corpus entry summary.
3. Mark `requires_review` when the relationship, event claim, or wording is not
   yet trusted.
4. Keep competitor capability notes segregated from user-facing reasoning.
5. Preserve graph/corpus version context in any downstream implementation note.

## Mandatory Review Checks

- Confirm the source can be summarized without copying protected text.
- Confirm source ids and content hashes exist before import or reasoning.
- Confirm summaries are concise and attributable.
- Confirm review flags survive any import or transformation step.
