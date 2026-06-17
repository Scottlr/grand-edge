---
name: grandedge-github-issue-loop
description: Autonomous loop for Scottlr/grand-edge that selects unblocked GitHub issues, implements them one at a time, opens PRs, merges, closes issues, and cleans branches until no eligible issues remain.
---

# GrandEdge GitHub Issue Implementation Loop

Use this skill to run a fully autonomous agent loop that selects, implements,
and closes GitHub issues in `Scottlr/grand-edge` one at a time until no
unblocked issues remain.

## Prerequisites

- GitHub CLI (`gh`) installed and authenticated.
- `gh auth status` succeeds.
- The current working directory is inside the `Scottlr/grand-edge` checkout.
- Read `AGENTS.md` before starting.

## Non-Negotiable Rules

- Only operate inside the current repository checkout and `Scottlr/grand-edge`.
- Use exactly one `codex/issue-<number>-<slug>` branch per issue.
- Never commit to, push to, or rewrite `main`, `master`, or the default branch.
- Never run `git add .`, `git add -A`, `git reset --hard`, `git clean -fd`,
  force-push, history rewrite, or permission-bypass commands.
- Stage only explicit files that belong to the selected issue.
- Do not stage generated caches, build outputs, logs, secrets, `target/`, or
  unrelated dirty work.
- Each issue must complete the full lifecycle: branch, implement, verify, PR,
  merge, issue close, remote branch delete, local branch delete.
- Between issues, the checkout must be on the default branch with no tracked
  or staged changes.
- If any step fails and cannot be recovered, stop and report the blocker.

## Loop Workflow

### Step 1: Discover the Repository

```powershell
gh repo view Scottlr/grand-edge --json nameWithOwner,url,defaultBranchRef --jq '{repo: .nameWithOwner, defaultBranch: .defaultBranchRef.name}'
git remote -v
git branch --show-current
git status --short
```

Store `Scottlr/grand-edge` and the default branch for all subsequent commands.

### Step 2: Select an Unblocked Issue

Use GitHub search with native dependency qualifiers to find an open issue that
has no blockers and is not already in progress:

```powershell
$REPO = "Scottlr/grand-edge"

$ISSUE_JSON = gh issue list `
  --repo $REPO `
  --state open `
  --search "is:issue is:open -is:blocked -label:in-progress -label:pr-taken" `
  --json number,title,url,body,labels,assignees,milestone,author,updatedAt `
  --limit 1

if (-not $ISSUE_JSON -or $ISSUE_JSON -eq "[]") {
  Write-Host "No unblocked issues remaining. Loop complete."
  exit
}

$ISSUE = $ISSUE_JSON | ConvertFrom-Json | Select-Object -First 1
$ISSUE_NUM = $ISSUE.number
```

This query returns the first open issue that:
- is not blocked by any other open issue (GitHub native `blocked-by` relationships)
- does not have the `in-progress` or `pr-taken` label (prevents two agents
  grabbing the same issue, even without an assignee)

### Step 3: Claim the Issue

Immediately label the issue as in-progress to prevent contention:

```powershell
gh issue edit $ISSUE_NUM --repo $REPO --add-label in-progress
```

### Step 4: Read the Issue Body

```powershell
gh issue view $ISSUE_NUM --repo $REPO --json number,title,url,body,labels,assignees,milestone,author,comments
```

Parse the issue body to extract:
- Description and expected behavior
- Acceptance criteria (explicit or inferred)
- Definition of Done (if provided)
- Any linked PRs, docs, screenshots, or failing commands
- Blocker metadata (`Blocked by:` sections)

### Step 5: Create the Issue Branch

Start from the repository default branch:

```powershell
$DEFAULT_BRANCH = gh repo view --repo $REPO --json defaultBranchRef --jq .defaultBranchRef.name
git fetch origin
git switch $DEFAULT_BRANCH
git pull --ff-only

$SLUG = <generate-short-slug-from-title>
$BRANCH = "codex/issue-${ISSUE_NUM}-${SLUG}"
git switch -c $BRANCH
```

The slug should be a lowercase, hyphenated, shortened version of the issue
title (max ~30 chars, no special characters).

### Step 6: Outline the Definition of Done

Before implementing, write down an explicit Definition of Done:

- Restate the issue's reported behavior or request.
- Copy or summarize existing acceptance criteria.
- Infer narrow acceptance criteria if none are provided.
- List files or modules likely involved.
- Identify tests or commands that should prove the fix.
- Note risks, migrations, or user-visible behavior changes.

Write the Definition of Done in the PR body draft or a temporary notes file.
Do not add permanent run artifacts unless the issue explicitly requires them.

### Step 7: Implement the Fix

Apply the smallest correct change that satisfies the Definition of Done.

Rules:
- Prefer existing code patterns over new architecture.
- Keep unrelated cleanup out of the branch.
- Preserve unrelated user changes in the worktree.
- Add or update tests when the issue changes behavior.
- Update docs, examples, schemas, or generated artifacts when affected.
- Do not hard-code issue-specific logic unless the repository is explicitly
  a one-off application and the issue requires it.

### Step 8: Verify

Run the most relevant verification first, then broaden if the risk justifies it.

Record:
- Commands run
- Pass/fail result
- Important failure text if something still fails
- Any checks skipped and why

Proof must map directly back to the Definition of Done.

Before committing, check `git status --short` and ensure only files belonging
to the selected issue are staged.

### Step 9: Commit

```powershell
git add <changed-files-explicitly>
git commit -m "fix: address issue #${ISSUE_NUM}"
git diff --stat
git diff --check
git status --short
```

Never use `git add .` or `git add -A`.

### Step 10: Push and Create PR

```powershell
git push -u origin $BRANCH

$PR_TITLE = "Fix #${ISSUE_NUM}: $($ISSUE.title)"
$PR_BODY_FILE = "$env:TEMP\\grandedge-pr-${ISSUE_NUM}.md"

gh pr create `
  --repo $REPO `
  --title $PR_TITLE `
  --body-file $PR_BODY_FILE
```

The PR body should include:
- Summary linking the issue
- Guardrails statement
- Definition of Done checklist
- Verification evidence
- Notes and risk

### Step 11: Wait for Checks

```powershell
gh pr checks --watch
```

If checks fail, diagnose and fix on the same branch, then push additional
commits. Repeat until checks pass or a non-fixable blocker is identified.

### Step 12: Merge the PR

```powershell
gh pr merge --squash --delete-branch
```

If the repository prefers merge commits or rebase merges, use the repository's
normal policy instead of `--squash`.

### Step 13: Close the Issue

Close the issue only after the PR is merged:

```powershell
gh issue close $ISSUE_NUM --repo $REPO --comment "Fixed by PR #<pr-number>."
```

### Step 14: Branch Cleanup

Delete the remote and local issue branches:

```powershell
git switch $DEFAULT_BRANCH
git pull --ff-only

# Remote branch should be deleted by --delete-branch on merge, but verify:
gh api repos/$REPO/git/refs/heads/$BRANCH 2>$null
if ($LASTEXITCODE -eq 0) {
  gh api repos/$REPO/git/refs/heads/$BRANCH -X DELETE
}

# Delete local branch
git branch -d $BRANCH
```

### Step 15: Loop Back

Return to Step 2 and select the next unblocked issue.

The loop terminates when `gh issue list` with the
`-is:blocked -label:in-progress -label:pr-taken` search returns zero results.

## Quick Reference Commands

### Select next unblocked issue (one-liner)

```powershell
gh issue list --repo Scottlr/grand-edge --state open --search "is:issue is:open -is:blocked -label:in-progress -label:pr-taken" --json number --jq '.[0].number'
```

### Claim issue

```powershell
gh issue edit <number> --repo Scottlr/grand-edge --add-label in-progress
```

### View issue

```powershell
gh issue view <number> --repo Scottlr/grand-edge
```

### Invariant check

```powershell
git status --short
git branch --show-current
```

## Error Handling

- If an issue cannot be reproduced, document why and close it with a comment.
- If verification cannot run, keep the result as blocked or partial and state
  the exact command that could not run.
- If permissions prevent merge, close, or branch deletion, stop and report the
  exact missing permission or failing command.
- Do not claim "fixed", "done", or "complete" unless the PR is merged, the
  issue is closed, and branch cleanup is done or blocked by a documented
  permission failure.

## Security

- Never paste secrets into issue comments, PR bodies, or logs.
- Redact tokens, cookies, private URLs, and customer data.
- Prefer links to existing CI runs over copying long logs.
- If a command needs elevated access, document the exact blocker instead of
  inventing a workaround.
