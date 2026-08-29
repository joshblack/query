# Distinguish logical modules from source origins

Date: 2026-08-29

The first analysis slice needs to connect written module references, parsed
facts, and source locations without making a parser or resolver detail part of
the public contract. Issue #5 establishes this boundary before implementation.

## Decision

Use a resolver-issued logical module identity, represented by an opaque Module
ID, and keep it distinct from both the written module specifier and the source
origin. A resolver receives the importer, the written specifier, and the
import kind, then returns one of:

- a resolved Module ID with its source origin;
- an external result; or
- a typed resolution failure.

Each logical module owns all parser facts for its source. Linking, which runs
after module facts exist, owns aliases, re-exports, final symbols, component
edges, route links, ambiguity, and cycle groups. A module may be reached by
multiple specifiers, and those specifiers may converge on the same Module ID.

### Impact

This preserves the difference between identity and provenance. A source origin
can change while a resolver still identifies the same logical module, and
several specifiers can refer to one logical module without duplicating its
facts. Parser output remains local and ownership is unambiguous, while linking
can resolve cross-module meaning.

The public result can expose stable domain IDs and source provenance without
exposing parser or graph library types. The trade-off is that a resolver is a
required boundary, and link resolution must explicitly represent unresolved,
ambiguous, external, and cyclic relationships rather than selecting a
convenient traversal result.

### Alternatives

- **Use source origins as module identity.** This makes provenance convenient,
  but aliases and multiple origins become identity problems and source
  movement leaks into every relationship.
- **Use written specifiers as module identity.** This preserves what was
  written, but cannot represent convergence and causes duplicate parser facts.
- **Let each parser result own linked facts.** This couples local parsing to
  cross-module resolution and makes aliases, re-exports, and cycles dependent
  on parse order.
- **Expose parser or graph library identities.** This reduces translation
  work, but makes the contract dependent on implementation libraries and their
  traversal or index behavior.

## Links and resources

- [Analysis contract](../analysis-contract.md)
- [JSON format v1](../json-format-v1.md)
- [Supported syntax](../supported-syntax.md)

## Footnotes

The resolver boundary does not include package resolution in the first slice.
