# dogma

Decision records linked to the changes they justify, enforced in CI.

> δόγμα — *that which seems good*; the formal word for an assembly's decision.

<p align="center">
  <img src="docs/doge.png" width="620"
       alt="Doge in sunglasses, captioned: such decision, much enforce, very traceable, wow, no cite, no merge.">
</p>

---

## 🔥 The problem

Teams that keep written specifications have no way to stop them drifting from
reality. Someone changes behaviour, forgets the spec, and the document quietly
stops describing the system. Everyone knows this happens; nobody can gate it.

The usual attempt is a CI check asserting the specs are up to date. It fails
immediately, because the property is violated *by design* while work is in
flight — a branch mid-implementation is supposed to be inconsistent. So the
check is red on every PR from the first commit to the last, and a pipeline
that is always red is a pipeline people stop reading.

Existing spec tools attack this with a filing step: a command run at the end of
a change that merges its spec edits into the living document. That relocates
the problem rather than solving it. On a team with code review there is no
correct moment to run it — during review, feedback invalidates it; after merge,
a bot has to commit to a protected branch; at approval, most forges have no
such trigger. And if the filing step is skipped, nothing notices.

## ⚖️ The rule

Dogma enforces one thing:

> **a commit that changes an enforced path must cite an accepted decision.**

That's the whole design. It is green during normal work — commits that touch
nothing enforced are unaffected — and it fails only when someone changes
something significant without recording why.

From that single edge you get a traceability graph in which every link is a
commit fact, so nothing can drift out of date:

```
decision ──cited by──> commits ──contain──> spec changes
                          └──────────────> code changes
```

| question | command |
|---|---|
| why is this line here? | `dogma why specs/auth.md:42` |
| what did that decision cause? | `dogma impact 26-09-02-git-is-the-lifecycle` |
| what did we agree and never build? | `dogma gaps` |
| is this branch compliant? | `dogma check main..HEAD` |

## 🔧 How it works

### Decisions

A decision is a markdown file with a short YAML frontmatter. It lives at a path
derived from its id:

```
.dogma/decisions/26/09/02-git-is-the-lifecycle.md     id: 26-09-02-git-is-the-lifecycle
```

**The id is the path.** Splitting `26-09-02-git-is-the-lifecycle` on `-` gives
you the directory and filename directly, so there is no lookup table that could
disagree with the filesystem. There is also no counter, which means two people
on separate branches can never collide over the next number — a well-known
annoyance with numbered ADRs.

The date makes ids self-describing in `git log`, and lets a slug be reused
years later without ambiguity.

### Status

```
proposed ──(edit during review)──> accepted
                                └─> rejected
```

Status is edited exactly once, by hand, in the file you're already reviewing.
Only `accepted` satisfies the gate.

**Supersession is not a status.** A later decision declares it:

```yaml
status: accepted
supersedes: 26-09-02-git-is-the-lifecycle
```

The superseded decision is never touched. It stays `accepted`, because it *was* —
decisions are events, and commits that cited it were correct at the time.
"Superseded" is derived by following the link backwards.

### The citation

The only mechanical link between a decision and the work it justifies is a
commit trailer:

```
Expire idle sessions after 8 hours

Decision: 26-09-02-session-lifetime
Refs: QUA-1574
```

Note what this means: **a decision never lists the specs it affects, and a spec
never cites its decision in its text.** Both would be hand-maintained derived
data — exactly the thing that rots. The join lives in git.

Issue trackers meet the same way. Git already carries `Refs:` trailers, so
`dogma impact` surfaces the tickets alongside the files without dogma knowing
your tracker exists.

### Code and specs

Code and specs are linked by **co-citation**: they changed together, in commits
citing the same decision. `dogma impact` shows both sides, and `dogma why`
works on source files as readily as on specs.

What this deliberately does *not* do is tell you that a particular function
implements a particular requirement. That needs annotations in code, and
annotations rot on the first refactor. Commit-level traceability is free and
durable; requirement-level correspondence is unsolved by every tool in this
space, and dogma doesn't pretend otherwise.

## 📁 Layout

Defaults, all configurable:

```
.dogma/
  config.toml                        optional
  decisions/
    26/09/02-git-is-the-lifecycle.md
specs/
  auth.md                            enforced — changing this needs a citation
src/
```

Only `.dogma/` belongs to the tool. Specs live wherever your team wants them,
because dogma doesn't own them — `enforce` is just a list of globs.

The tool has no concept of a "spec". It knows *enforced paths*, and "spec" is
your word for whatever you chose to enforce.

## 🐕 Commands

```
dogma new <title>            create a decision, dated today, status: proposed
dogma list                   decisions and their statuses, oldest first
dogma check [range]          the gate
dogma gaps                   where the record and reality have come apart
dogma why <file>[:<line>]    what decided this
dogma impact <id>            what this decided
```

`--help` is the documentation. Dogma writes nothing into your repository — no
generated instruction files, no per-editor adapters — so there is nowhere else
for its behaviour to be described.

### `check`

The gate. Verifies three things:

1. every commit in the range touching an enforced path cites a decision
2. every cited decision exists and is `accepted`
3. the decisions directory is well-formed, and every `enforce` pattern matches
   something — a typo there would otherwise disable the gate *while leaving it
   green*, which is the worst failure a gate can have

```
$ dogma check main..HEAD
✓ decisions well-formed (12)
✓ enforce patterns match
✓ 3 enforced changes cite accepted decisions

  14 enforced files have no decision behind them  → dogma gaps
   2 accepted decisions are unimplemented         → dogma gaps
```

| exit | meaning |
|---|---|
| 0 | every enforced change cites an accepted decision |
| 1 | a violation |
| 2 | usage or environment error |

A decision's status is read at the **head of the range**, not at the citing
commit. So proposing and implementing in one PR works: the decision goes in as
`proposed`, is flipped to `accepted` during review, and the tree is coherent at
merge. The gate asks whether *this state* holds together, not whether every
intermediate commit did.

### `gaps`

Always exits 0. This is a report, never a gate — and that is deliberate. Every
repo adopting dogma has pre-existing files with no decision behind them, and
folding those into `check` would recreate the permanent-red problem the tool
exists to avoid. `check` surfaces the counts and points here; `gaps` does the
listing.

## ⚙️ Configuration

Optional. `.dogma/config.toml` or `dogma.toml`:

```toml
decisions = ".dogma/decisions"
enforce   = ["specs/**", "!specs/drafts/**"]
trailer   = "Decision"
```

An unknown key is an error rather than a silent no-op, for the reason above.

`enforce` patterns are evaluated in order and **the last match wins**, the same
rule `.gitignore` uses, so `!pattern` carves exceptions out of a broader sweep.

Nothing about the mechanism is spec-specific. Point `enforce` at `schema/`,
`proto/`, or `infra/terraform/` and it behaves identically. Enforcing the whole
repository is a legitimate configuration:

```toml
enforce = ["**", "!vendor/**", "!**/*.generated.rs"]
```

At that setting decisions stop being per-change paperwork and become durable
architectural anchors — one decision cited by years of commits. It is also what
traceability standards like DO-178C and IEC 62304 require, which heavyweight ALM
suites exist to provide.

**The decisions directory is never enforced**, whatever the config says. A
repository that enforced its own decisions could not add one: the commit would
need to cite an accepted decision, and a new decision is born `proposed`. The
exclusion is in code rather than left to the user, so the lockout is impossible
rather than one typo away.

**Not configurable:** the statuses, and which of them satisfies the gate. If
those varied per repo, `dogma check` would mean something different in each
one.

## 🤖 Working with agents

Decision records turn out to be a good fit for LLM-assisted development, for
two reasons beyond governance.

**The cost objection disappears.** The historical problem with decision records
is that nobody writes them. When the agent that reasoned through a choice
drafts the record from that same session, the marginal cost is near zero — the
reasoning already exists, it just needs somewhere durable to land.

**They transfer context between sessions.** An agent session ends and its
reasoning evaporates. Decisions live in the checkout, so any agent with the
repo can read them — offline, greppable, no API. `dogma why specs/auth.md`
returns the reasoning behind the thing an agent is about to modify.

One caveat worth designing against: a hollow record written by a human is two
terse lines you can spot. A hollow record written by a model is three fluent
paragraphs that restate the change and decide nothing. The scaffold's sections
— *Alternatives*, and *what would make us revisit* — are chosen to be
conspicuous when empty.

## 🚫 What it deliberately does not do

- **Stores nothing.** No index, no lockfile, no cache. Every answer is derived
  from git and the working tree at the moment you ask, so no state can go stale.
- **Writes nothing into your repo.** Beyond decisions you asked it to create.
- **Enforces no format.** The only thing it parses is a decision's frontmatter,
  because that is the only thing it reads. Specs are written however your team
  likes — prose, tables, Gherkin, diagrams.
- **Manages no lifecycle.** Git is the lifecycle. Branches are proposals, merges
  are acceptance, history is the archive. Two changes touching the same
  requirement produce a merge conflict, surfaced by git and resolved by a human,
  which is correct.
- **Integrates with nothing.** No tracker APIs, no forge plugins, no per-tool
  adapters.

See [`.dogma/decisions/26/09/02-git-is-the-lifecycle.md`](.dogma/decisions/26/09/02-git-is-the-lifecycle.md)
for the reasoning, including the alternatives that were rejected.

## 🧭 Coming from OpenSpec

Dogma exists because of a specific problem in [OpenSpec](https://github.com/Fission-AI/OpenSpec),
and if you're arriving from there, this is the disagreement in full.

OpenSpec keeps a `changes/<name>/` directory per change, holding delta specs —
`## ADDED` / `## MODIFIED` blocks — which `openspec archive` later folds into
the main specs while moving the folder into `changes/archive/`.

That directory is **two things at once**: a *record* of why something was done,
and a pending *operation* waiting to be applied. Everything else follows from
that conflation:

- An operation must be applied exactly once, so it needs a moment — and on a
  team with code review, no moment works. During review, feedback invalidates
  the fold and there's no un-archive. After merge, a bot commits to a protected
  branch. At approval, most forges have no trigger.
- Because the moment can't be scheduled, the discipline can't be gated in CI
  without the check being red for the entire life of every change.
- And if an operation is never retired, it stays live. Two changes touching the
  same requirement then fight: re-applying the older one overwrites the newer
  text, and the check sticks red with no way to clear it.

Dogma's answer is to stop having operations. Specs are edited directly; git
carries the diff, the ordering, the merge and the history. Decisions are a
separate append-only record, joined to the work by a commit trailer.

| | OpenSpec | dogma |
|---|---|---|
| spec edits | deltas in a change folder, folded later | edited directly, in the branch |
| history | `changes/archive/` | git |
| ordering & merge | the fold engine | git merge, conflicts included |
| lifecycle state | `status`, directory position | none — branches and merges are the lifecycle |
| the filing step | `openspec archive` | doesn't exist |
| what it stores | specs, changes, archive | decisions only |

**What OpenSpec has that dogma does not**, and these are real:

- **A guided authoring workflow.** The propose → specs → design → tasks artifact
  graph, with schema-driven instructions per phase. That's the bulk of what
  OpenSpec *is*, and dogma has no equivalent — it assumes you know what you want
  to write.
- **A spec grammar and validator.** Requirements, scenarios, and a strict mode.
  Dogma parses nothing but a decision's frontmatter, deliberately.
- **Multi-repo stores.** Planning that lives in its own repo, referenced by code
  repos.
- **AI tool adapters.** Generated skills and instructions for a dozen editors.

If you want the guided workflow, use OpenSpec — the two aren't competing for the
same job. Dogma does one thing: it makes sure consequential changes carry a
recorded reason, and lets you ask about it afterwards.

You can also use both. `openspec validate --specs --strict` runs standalone with
no `changes/` directory, so nothing stops you keeping OpenSpec's spec grammar
and letting git handle the lifecycle.

The upstream discussion is
[Fission-AI/OpenSpec#1683](https://github.com/Fission-AI/OpenSpec/issues/1683)
and [#1684](https://github.com/Fission-AI/OpenSpec/pull/1684), which proposed
fixing this inside OpenSpec before we concluded the fix was a different tool.

## 🚧 Status

Early. The CLI surface is settled and `new` works; `check` is next, then the
query commands.

## 📜 Licence

MIT OR Apache-2.0
