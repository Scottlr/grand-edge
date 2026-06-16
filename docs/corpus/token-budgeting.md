# Token Budgeting

Corpus work must stay route-first and sample-limited.

## Default Limits

- Open at most `3` source files per route.
- Sample at most `5` entries per route.
- Read router docs before any corpus JSON.
- Use `rg` or equivalent targeted search before opening a source file.
- Never paste full corpus JSON arrays into chat or skill references.

## Escalate Instead Of Broad Loading

Stop and ask when:

- the task appears to need more than `3` source files
- the task appears to need more than `5` sample entries
- review status or licensing is unclear
- a route does not map cleanly to the request
- the request asks for end-user copy derived from competitor or unreviewed notes

## Reporting

When corpus context affects an answer or implementation, report:

- route id used
- source ids consulted
- whether entries required review
- graph/corpus version mentioned by the source
