# 🐕 dogma

Decision records linked to the changes they justify, enforced in CI.

> δόγμα — *that which seems good*; the formal word for an assembly's decision.

Written specifications drift from reality, and the discipline that would stop
them can't be gated in CI without turning every open pull request red.

*Arriving from [OpenSpec](https://github.com/Fission-AI/OpenSpec)? See
[why dogma diverged](#coming-from-openspec).*

<p align="center">
  <img src="docs/doge.png" width="620"
       alt="Doge in sunglasses, captioned: such decision, much enforce, very traceable, wow, no cite, no merge.">
</p>

---

<a id="design"></a>

## ⚖️ Design

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

### Tracing code, optionally

Nothing in the mechanism is spec-specific — `enforce` is a list of globs, and
the tool has no idea what a spec is. That makes the interesting setting a
choice rather than a feature:

```toml
enforce = ["specs/**"]                      # spec-level traceability
enforce = ["**", "!vendor/**"]              # every line in the repository
```

Enforce only your specs and you get the modest version: consequential documents
can't change without a reason. Enforce code as well and something better falls
out — **every line in the repository traces to a decision**:

```console
$ dogma why src/session.rs:88
26-09-02-session-lifetime  (accepted)  Expire idle sessions after 8 hours
```

At that setting decisions stop being per-change paperwork and become durable
architectural anchors: one decision, cited by years of commits, answering "why
is this like this" for everything downstream of it. Because the link is a commit
trailer rather than an annotation in the code, it survives refactors — git
already tracks lines across moves and renames.

It is also, incidentally, what traceability standards demand. DO-178C in
aerospace and IEC 62304 in medical devices both require every change to be
traceable to an approved change request; teams satisfy that today with six-figure
ALM suites whose core function is this one edge.

**The honest cost at that setting** is chore pressure. Dependency bumps,
formatting passes and generated files have no decision behind them, so you end
up either growing the exemption list or minting a routine-maintenance decision
that everything cites — a rubber stamp wearing a lanyard. Enforcing the
genuinely consequential surfaces (specs, schemas, public APIs, migrations,
infrastructure) is where "changed silently" actually costs you something.

## 🔧 Concepts

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

## ⌨️ Commands

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

Point `enforce` at `schema/`, `proto/`, or `infra/terraform/` and it behaves
identically — see [Design](#design) for what enforcing the whole repository
buys you.

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

<a id="coming-from-openspec"></a>

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
