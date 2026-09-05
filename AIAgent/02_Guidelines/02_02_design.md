# Design guideline

This guideline defines the mandatory structure for every design document produced in the Design Definition Phase (folder '04_DesignDev', finalised into '04_Design') for *product*, in addition to the general rules of '01_01_management.md' (§ Document rules). It is tailored to *product* being built in Haskell (see '00_agents.md').

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [AD01] | 01_01_management.md | Management requirements |
| [AD02] | 02_01_requirements.md | Requirements guideline |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | The software product being developed; formally defined by the Requirements specification (folder '03_Requirements'). |

## 1. Purpose and scope

This guideline applies to all design documents for *product*. It does not itself contain design; it defines how design documents must be structured and worded.

## 2. Mandatory document structure

Every design document must contain the following sections, in this order. A section that has no content must still be present, with its body set to "- none -" — sections are never omitted.

1. **Header** — product name and a one-line reference to this guideline.
2. **Applicable documents** — as required by '01_01_management.md' (§ Document rules → Applicable documents).
3. **Terms** — as required by '01_01_management.md' (§ Document rules → Terms), plus any term newly introduced by the design.
4. **Scope** — one short paragraph stating what part of *product* the document covers, and one stating what is explicitly out of scope.
5. **Architecture overview** — the module decomposition of *product* (§3 below).
6. **Type declarations** — the data model of *product*, in actual Haskell syntax (§4 below).
7. **Function signatures** — the public interface of each module, as type signatures only (§5 below).
8. **Data flow** — the processing pipeline of *product*, naming the type that flows between stages.
9. **Error handling strategy** — how expected failure is represented and propagated (§7 below).
10. **Purity and effect boundaries** — which parts of *product* are pure and which perform I/O (§8 below).
11. **Traceability** — how design elements reference the requirements they realise (§9 below).
12. **Open questions** — permitted only while the document is in Dev status; must be empty before finalisation (finalisation checklist item 2: "No placeholders or TODOs remain").

## 3. Architecture overview

- Every module of *product* is listed with its full module path (e.g. `Genc3.Parser`), a one-sentence statement of its responsibility, and the modules it depends on (imports).
- Module dependencies are acyclic. Haskell's compiler rejects a cyclic import, so a design that describes one is invalid by construction; this is a normal consistency check, not an extra rule to remember separately.
- Dependencies are shown as a table with columns **Module**, **Responsibility**, **Depends on**.

## 4. Type declarations

- Every type central to the design (`data`, `newtype`, `type`, or type class) is declared using actual Haskell syntax, not a prose paraphrase.
- Each type declaration is immediately followed by a comment naming the requirement ID(s) it realises, e.g.:

  ```haskell
  -- realises FR-002, NFR-001
  data Token = TWord Text | TNumber Integer | TSymbol Char
    deriving (Eq, Show)
  ```

- A type that models a domain concept already defined as a term (per '01_01_management.md' § Document rules → Terms) references that term by its italicised name rather than re-explaining it in the type's own documentation.

## 5. Function signatures

- Every function that forms a module's public interface is declared by its top-level type signature only — no implementation — e.g.:

  ```haskell
  -- realises FR-003
  parseIncremental :: Config -> ByteString -> ParseState -> (ParseState, [Token])
  ```

- The requirement-ID comment is mandatory when the function realises a functional requirement (§4.1 of '02_01_requirements.md'); it is optional for a purely internal helper that realises no requirement directly.

## 6. Data flow

The processing pipeline of *product* is documented as an ordered sequence of stages (e.g. stdin → lexer → incremental parser state → output), naming the type that flows between each stage. Each stage references the module (§3) that implements it.

## 7. Error handling strategy

- All expected failure is represented in types — `Either`, `Maybe`, or a dedicated error type — never by partial functions (e.g. `head`, `fromJust`) or uncaught runtime exceptions.
- An error type's constructors are declared per §4 and traced to the requirement(s) describing the corresponding failure behaviour.

## 8. Purity and effect boundaries

- Parsing and data-transformation logic is pure; effects (I/O, environment access, exceptions) are confined to the `app` executable's entry point (`app/Main.hs`) and to module boundaries that are explicitly marked as effectful.
- For every function declared per §5, the design states whether it is pure or performs I/O, either by grouping functions under a "Pure" / "Effectful" subheading or by an inline note.

## 9. Traceability

- Every design element (module, type, or function signature) that realises one or more requirements states those requirement IDs directly at its declaration (§4, §5), keeping requirement → design traceability explicit at the point of use.
- Before finalisation, every requirement categorised as Functional or Interface ('02_01_requirements.md' §4.1) is realised by at least one design element; this is checked as part of the finalisation checklist's consistency item ('01_01_management.md', checklist item 3).

## 10. Template

````
# Design

Product name: *<name>*

This design follows the structure defined in '02_02_design.md'.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [AD01] | 01_01_management.md | Management requirements |
| [AD02] | <requirements specification file name> | Specification |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | (see Requirements specification, § Terms — not redefined here) |
| *<name>* | <context (optional)> | <definition> |

## Scope

Covers: ...
Out of scope: ...

## Architecture overview

| Module | Responsibility | Depends on |
| --- | --- | --- |
| `Genc3.<Name>` | ... | ... |

## Type declarations

```haskell
-- realises FR-...
data <Name> = ...
```

## Function signatures

```haskell
-- realises FR-...
<name> :: ...
```

## Data flow

stdin → ... → stdout

## Error handling strategy

...

## Purity and effect boundaries

Pure: ...
Effectful: ...

## Traceability

...

## Open questions

...
````
