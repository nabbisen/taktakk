# taktakk RFCs

Design notes for taktakk features and policies. Each RFC scopes
one piece of work in enough detail that an implementer can start
without a second design pass — but no more than that.

These are not blanket commitments. The [ROADMAP](../ROADMAP.md)
sets which of these will actually ship and in what order. An RFC
landing here means the design is settled enough to write code
from; not landing here means the design is still soft.

## How this directory works

The lifecycle is governed by
[RFC 000 — RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md).
Briefly:

- **`proposed/`** — open for review and discussion. Implementer
  should not yet start work; the design may change.
- **`done/`** — implemented and shipped. The RFC is now a
  historical record of the design decisions.
- **`archive/`** — withdrawn or superseded. Preserved as
  evidence the design was considered.

Files do not move out of `done/` or `archive/` after they land
there. Numbering is permanent: a file's RFC number is assigned
at creation and never changes, even if the file moves between
folders.

## Implementation order

Within `proposed/`, RFCs are listed by intended work sequence,
not by RFC number. The numbering reflects the order RFCs were
written; the order above reflects the priority an implementer
should pick them up.

## Template

The standard shape is light:

```markdown
# RFC NNN — Title

**Status.** Proposed | Implemented (vX.Y.Z) | Withdrawn | Superseded by RFC NNN
**Tracks.** ROADMAP item or other context this addresses.
**Touches.** crates / modules the work lands in.

## Summary

One paragraph. What changes for the user, why now, why this shape
over the alternatives.

## Background (optional)

Context the implementer needs that isn't on ROADMAP.md. Skip when
the title alone tells you what's going on.

## Design

What the implementer builds. Schemas, function signatures, state
machines, error paths. Treat this as the contract.

## Multiple implementation steps

If the work splits into stages that can ship separately, list them
here with rough scope.

## Tests (when non-trivial)

What the implementer should write to call it done.

## Security considerations (when applicable)

What an attacker might try, and what the design does about it.

## Open questions

Anything the implementer should bring back before merging.
```

### When to add the heavier sections

The light template handles small, mechanical items. Anything
medium or larger — schema changes, new background workers,
cross-cutting policies, third-party integration shapes — earns
the heavier sections:

- **Requirements** — explicit list of what must be true after the
  change ships, separately from the design that delivers it.
- **Design** (replaces "Design" section title above) — same
  intent, but expected to be thorough rather than sketchy.
- **Test plan** — coverage map: what unit, integration, and
  regression tests get added; what existing tests might need to
  move.
- **Security considerations** — first-class section, not a footnote.

Each RFC declares which sections it carries by the headings it
uses. There's no separate metadata.

## Process

The full lifecycle is described in
[RFC 000 — RFC lifecycle policy](./done/000-rfc-lifecycle-policy.md). The short version:

1. New RFC: open a draft as `rfcs/proposed/NNN-slug.md` with
   Status `Proposed`. The number is the next unused integer,
   zero-padded to three digits, and never reused.
2. Iterate in review until the design is settled.
3. When the work ships, move the file to `rfcs/done/`, update
   Status to `Implemented (vX.Y.Z)`, and update inbound
   references in this README and other RFCs.
4. RFCs that don't pan out move to `rfcs/archive/` with Status
   `Withdrawn` (and a one-line reason) or `Superseded by RFC NNN`.
   They stay there as a record.

Files are never deleted. The full reasoning is in RFC 000.
