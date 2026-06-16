# Route Examples

## `mechanical_relation_edge`

- Use for item-set pack/unpack questions.
- Search first: `rg -n "item_set|repair|alchemy|dose|degrade" data/relations`

## `market_event_context`

- Use for updates, events, activity shifts, and event-linked baskets.
- Search first: `rg -n "game_update|event_hypothesis|affected_item_ids" data/corpus`

## `source_policy_review`

- Use for licensing, copied-text risk, or review-state questions.
- Search first: `rg -n "license|content_hash|requires_review" docs/corpus data/corpus data/relations`
