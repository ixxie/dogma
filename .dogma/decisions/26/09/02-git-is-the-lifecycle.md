---
status: accepted
title: Git is the lifecycle
---

## Context

Spec tools tend to grow a parallel version-control system inside a directory:
change folders holding deltas, a merge step that folds them into the living
specs, an archive folder for finished work, and a state field tracking where
each change has got to.

That structure has a specific failure. A change folder is two things at once —
a *record* of why something was done, and a pending *operation* waiting to be
applied. An operation must be applied exactly once, which means it needs a
moment; and on a team with code review there is no correct moment. Applying it
during review means feedback invalidates it. Applying it after merge means a
bot committing to a protected branch. So the discipline cannot be enforced in
CI without the check being red for the entire life of every change, and a
pipeline that is always red is one people stop reading.

Worse, if the operation is never retired, it stays live forever. Two changes
that touch the same requirement then fight: re-applying the older one
overwrites the newer text, and the check gets stuck red with no way to clear
it.

## Decision

There are no deltas and no lifecycle state. Specs are edited directly, in the
branch, like any other file. Git carries the diff, the ordering, the merge, and
the history.

Decisions are a separate, append-only record. The only link between a decision
and the change it justifies is a commit trailer:

    Decision: 26-09-02-git-is-the-lifecycle

The tool enforces one rule — a commit touching a guarded path must cite an
accepted decision — and answers questions by traversing that link in git.

## Alternatives

**Fold deltas into specs, with base snapshots.** Keep change folders, but
record the state each delta was written against so re-application can do a
three-way merge. This is the honest fix for the replay problem, and it is what
upstream OpenSpec would need to build. Rejected because it is reconstructing
merge-base and three-way merge — machinery git already has, correct, for free.

**Fold, and gate on task completion.** Trigger the merge when a change's tasks
are all ticked. Rejected because the operation still never retires, so
superseded deltas still fight; the fix only bounds the window rather than
closing it, and the bound depends on the filing step that has no good moment.

**Annotate code with spec references.** Would give requirement-level
traceability rather than commit-level. Rejected because annotations rot on the
first refactor, and maintaining them by hand is the class of drift this design
exists to remove.

## Consequences

Easier: nothing to schedule, nothing stored, no state that can go stale. Two
changes touching the same requirement produce a merge conflict — surfaced by
git, resolved by a human, which is correct. Every question is answered from
commits, so no index can disagree with reality.

Harder: guarded content cannot change without a recorded reason. That is the
point, and it is also the whole cost. If writing a decision is heavyweight,
teams will write hollow ones to pass the gate, so the template must stay short.

Traceability is commit-level, not requirement-level. `dogma why` can say which
decision a line came from; nothing verifies that code does what a spec says.
That correspondence problem is unsolved by every tool in this space, and this
one does not claim otherwise.

We would revisit this if git ceased to be the universal substrate — if specs
had to live somewhere without version control, the argument collapses and a
homemade lifecycle becomes necessary again.
