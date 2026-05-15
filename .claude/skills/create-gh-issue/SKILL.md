---
name: create-gh-issue
description: Use when user mentions finding a bug or issue, says they want to implement a feature, invokes /create-issue, or when you identify an out-of-scope problem to defer rather than address mid-task.
---

# Create GitHub Issue

## Overview

Create a GitHub issue, assign it to yourself, and create a branch from it. All three steps are required — never skip any of them.

## Process

### Step 1: Collect title and description

If the user described the issue clearly, extract title and description from context. For out-of-scope issues you discovered yourself, draft both and confirm with the user before creating.

### Step 2: Ask for a label

**Always ask** — do not skip because "no labels are established" or "wrong labels would be noise." Use AskUserQuestion with these options (all are default GitHub labels present on any new repo):

- `bug` — Something is broken or behaves incorrectly
- `enhancement` — New feature or improvement to existing functionality
- `documentation` — Docs, comments, or spec updates

If the repo has custom labels (e.g. `refactor`), include them. Run `gh label list` first if unsure.

### Step 3: Create the issue

```bash
gh issue create \
  --title "<title>" \
  --body "<description>" \
  --label "<label>" \
  --assignee @me
```

`--assignee @me` is always required. Never omit it.

### Step 4: Create a branch from the issue

```bash
gh issue develop <issue-number> --checkout
```

Always do this immediately after the issue is created. If the current branch has uncommitted changes, stash or commit them first.

## Common Rationalizations to Ignore

| Rationalization | Reality |
|---|---|
| "No labels are established on this repo" | The standard labels above work on any repo |
| "No clear assignee convention" | Always use `--assignee @me` |
| "User asked for an issue, not a fix" | Branch creation is part of issue creation — always do it |
| "Current branch has uncommitted changes" | Stash or commit first, then create the branch |
| "I'll skip the branch since I'm mid-task" | Create the branch, then return to the original task |
