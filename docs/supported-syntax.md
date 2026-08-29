# Supported syntax

This document defines the first-slice syntax boundary. It describes what can
produce facts or links, and what must produce an explicit incomplete outcome.

## Source files

The first slice analyzes UTF-8 files with these extensions:

- `.js`
- `.jsx`
- `.ts`
- `.tsx`

Ordinary syntax that is unrelated to the supported forms does not make an
analysis incomplete.

## Imports

Static ESM imports are supported when the module specifier is a string
literal. The supported forms are:

- side-effect imports;
- default imports;
- named imports;
- named import aliases;
- namespace imports;
- declaration-level type-only imports; and
- specifier-level type-only imports.

## Exports

The supported export forms are:

- exported declarations;
- named local exports;
- named local export aliases;
- default declarations;
- default expressions represented by a source-backed identity;
- named re-exports;
- named re-export aliases;
- star re-exports;
- namespace re-exports; and
- type-only exports and re-exports.

## Components

A component is supported when it is a PascalCase function declaration, or a
PascalCase variable initialized by an arrow function or a function expression.

Component edges are created for JSX identifier elements and statically
resolvable JSX member elements. Intrinsic elements and fragments do not create
component edges. The canonical direction is from the component that renders to
the component that is rendered.

## Routes

The parser records a route candidate for a direct `jsonRoute` call with:

- one object-literal argument;
- one literal path;
- an identifier or static member component; and
- any property order, including component shorthand.

Linking recognizes a route declaration only when the callee resolves to a
source-qualified symbol named `jsonRoute`. A local spelling of `jsonRoute` is
not sufficient. The component expression must also resolve to one
source-qualified component symbol.

The following outcomes are explicit:

- a shadowed callee is not recognized;
- an unresolved callee remains an unresolved route candidate;
- an aliased callee is unsupported unless the first-slice resolver and linker
  can prove the source-qualified binding;
- a member callee is not recognized unless it resolves to the source-qualified
  symbol;
- an indirect callee is not recognized; and
- a computed callee is not recognized.

Duplicate route declarations are preserved and reported as an ambiguity. A
literal path with an unresolved component remains an incomplete route
candidate, not a recognized route or a silently discarded call.

## Unsupported relevant forms

Relevant unsupported syntax produces an explicit incompleteness diagnostic.
The first slice does not support:

- CommonJS;
- dynamic `import()`;
- computed module specifiers;
- template route paths;
- spread or computed route properties;
- indirect or computed route factory calls;
- `React.createElement`;
- computed JSX members;
- higher-order components;
- `memo`;
- `forwardRef`;
- class components; and
- package resolution.
