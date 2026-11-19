# Standard kickoff for every implementation session

Every phase prompt includes these steps. Do them **before writing any code**.

## Step 1: Orientation check (fresh eyes)
Before relying on the work of earlier phases, check that the starting state matches what the spec and `docs/STATUS.md` claim:
- `main` builds and CI on `main` is green (or say what's red).
- The deliverables of the phases you depend on exist and do what STATUS says, **for the code your phase will touch or build on**. This is a sanity check, not a full review: each PR already had a review session.
- `docs/DECISIONS.md` entries that affect your phase are understood.

Report anything that's missing, broken, or contradicts the spec. Fix nothing yet.

## Step 2: Plan and questions
Write, in chat:
1. **Your understanding** of the phase in a few sentences: what you'll deliver, and what's explicitly out of scope.
2. **Your plan:** ordered steps, the files/crates you'll create or change, new dependencies (and why), how you'll test it.
3. **Questions**, but only where the spec is ambiguous or contradictory *and* the answer would change what you build. For minor choices, pick a sensible default, list it under "Assumptions", and record it in `docs/DECISIONS.md` later if it matters.
4. **Orientation findings** from Step 1.

Then **stop and wait for the owner's answers and go-ahead.** The owner may relay questions to the orchestrator, so make each question self-contained (quote the spec section).

Tip for the owner: starting the session in Claude Code's **plan mode** (Shift+Tab) enforces this pause, because the session can't edit files until you approve the plan.

## Step 3: Implement
Follow the plan. If you hit a new ambiguity mid-phase that would change the outcome, stop and ask rather than guess.

## Step 4: Before opening the PR
Run fmt/clippy/tests, update `docs/STATUS.md` and `docs/DECISIONS.md`, write the manual test checklist `docs/testing/phase-<id>.md`, and open the PR. A separate review session with clean context reviews it afterwards (`docs/prompts/review-pr.md`). Don't review your own PR as a substitute.
