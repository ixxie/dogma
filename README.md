# dogma

Decision records linked to the changes they justify, enforced in CI.

> δόγμα — *that which seems good*; the formal word for an assembly's decision.

## What it does

One rule:

> **a commit that changes a guarded path must cite an accepted decision.**

That's it. From that single invariant you get a traceability graph where every
edge is a commit fact, so nothing can drift:

```
decision ──cited by──> commits ──contain──> spec changes
                          └──────────────> code changes
```

| question | command |
|---|---|
| why is this line here? | `dogma why specs/auth.md:42` |
| what did that decision cause? | `dogma impact 26-08-24-session-lifetime` |
| what did we agree and never build? | `dogma unbuilt` |
| is this branch compliant? | `dogma check main..HEAD` |

## What it deliberately doesn't do

- **Stores nothing.** No index, no lockfile, no cache. Every answer is derived
  from git and the working tree at the moment you ask, so there is no state
  that can go stale.
- **Writes nothing into your repo.** No generated instruction files, no editor
  configs, no per-tool adapters. `--help` is the documentation.
- **Enforces no format.** The only thing it parses is `status:` in a decision's
  frontmatter, because that's the only thing it reads. Specs are written however
  your team likes — prose, tables, Gherkin, diagrams.
- **Manages no lifecycle.** Git is the lifecycle. Branches are proposals, merges
  are acceptance, history is the archive.

## Layout

Defaults, all configurable:

```
dogma/
  decisions/
    26/08/24-session-lifetime.md     status: accepted
  specs/
    auth.md                          guarded — changing this needs a citation
src/
```

A decision id *is* its path: `26-08-24-session-lifetime` resolves to
`26/08/24-session-lifetime.md`. No counter, so concurrent branches can never
collide over the next number.

Citing one is a commit trailer:

```
Add session expiry to the auth spec

Decision: 26-08-24-session-lifetime
```

## Configuration

Optional. `dogma.toml`, `dogma/config.toml`, or `.dogma/config.toml`:

```toml
decisions = "dogma/decisions"
guarded = ["dogma/specs/**"]
trailer = "Decision"
accepted_states = ["accepted"]
```

Nothing about the mechanism is spec-specific — point `guarded` at `schema/`,
`api/`, or `infra/terraform/` and it works the same way.

## Status

Early. The CLI surface is settled; the commands are being filled in.

## Licence

MIT OR Apache-2.0
