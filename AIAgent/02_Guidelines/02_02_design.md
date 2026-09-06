# Design guideline

This guideline defines the mandatory structure for every design document produced in phase 4 *Design Definition* (folder '04_Design') for *product*, in addition to the general rules of '01_01_management.md' (§ Document rules). It is tailored to *product* being built in C++ with the STL and Qt (see '00_agents.md').

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | 01_01_management.md | Management requirements |
| [2] | 02_01_requirements.md | Requirements guideline |

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
5. **Architecture overview** — the component decomposition of *product* (§3 below).
6. **Type declarations** — the data model of *product*, in actual C++ syntax (§4 below).
7. **Function signatures** — the public interface of each component, as declarations only (§5 below).
8. **Class diagram** — every class of *product* and its relationships to the others (§6 below).
9. **User interface design** — every window of *product*, its widgets and its layout (§7 below).
10. **Data flow** — the processing pipeline of *product*, naming the type that flows between stages.
11. **Error handling strategy** — how expected failure is represented and propagated (§9 below).
12. **Side-effect boundaries** — which parts of *product* are free of side effects and which perform I/O (§10 below).
13. **Traceability** — how design elements reference the requirements they realise (§11 below).
14. **Open questions** — permitted only while the change cycle is running; must be empty before the agent asks gate question G ('01_01_management.md' § Change cycle → Gate).

## 3. Architecture overview

- Every component of *product* is listed with its C++ namespace (e.g. `genc3::parser`), the header and source files that make it up, a one-sentence statement of its responsibility, and the components it depends on.
- Component dependencies are acyclic. A C++ build does not reject a cycle, so acyclicity is a rule of this guideline rather than a property enforced by the compiler: a cycle makes a component impossible to read, test, or replace on its own.
- Dependencies are shown as a table with columns **Component**, **Files**, **Responsibility**, **Depends on**.

## 4. Type declarations

- Every type central to the design (`struct`, `class`, `enum class`, or `using` alias) is declared using actual C++ syntax, not a prose paraphrase. The declaration carries no member bodies: it states the type, not its implementation.
- An abstraction over several types is declared either as an abstract base class with pure virtual member functions, or as a concept, whichever the design intends. The choice is stated, because it decides between run-time and compile-time dispatch.
- Each type declaration is immediately preceded by a comment naming the requirement ID(s) it realises, e.g.:

  ```cpp
  // realises FR-002, NFR-001
  enum class TokenKind { Word, Number, Symbol };

  // realises FR-002
  struct Token
  {
      TokenKind kind;
      QString   text;
  };
  ```

- A type that models a domain concept already defined as a term (per '01_01_management.md' § Document rules → Terms) references that term by its italicised name rather than re-explaining it in the type's own documentation.

## 5. Function signatures

- Every function that forms a component's public interface is declared by its declaration only — no definition — e.g.:

  ```cpp
  // realises FR-003
  ParseResult parseIncremental(const Config& config,
                               const QByteArray& input,
                               ParseState& state);
  ```

- A declaration states `const` and reference or value passing as the design intends them, because both carry design meaning: what the function may change, and what it copies.

- The requirement-ID comment is mandatory when the function realises a functional requirement (§4.1 of '02_01_requirements.md'); it is optional for a purely internal helper that realises no requirement directly.

## 6. Class diagram

- Every class and every `struct` declared per §4 appears in a class diagram in the section 'Class diagram'. A type that is declared but shown in no diagram is a gap in the design, not an omission of the diagram.
- A diagram shows, for each class: its name, the attributes and operations that carry design meaning, and its relationships to the other classes — inheritance, composition, aggregation, association — with the multiplicities wherever they are not one to one.
- Where the classes of *product* do not fit one readable diagram, the section holds several, each named after the component (§3) it covers. A class that appears in more than one diagram carries its attributes and operations in exactly one of them, and is shown by name only in the others.
- A diagram is given as Mermaid `classDiagram` source, so that it is versioned as text alongside the design and rendered where the design is read, e.g.:

  ```mermaid
  classDiagram
      class Token {
          +TokenKind kind
          +QString text
      }
      class Lexer {
          +Token next()
      }
      Lexer --> Token : produces
  ```

- The diagram does not repeat the requirement IDs of §4 and §5. Traceability stays at the declarations (§11).

## 7. User interface design

Where *product* presents a user interface, the design states it, so that the interface is designed rather than left to the files a designer tool generates.

### 7.1 Widget tree

Every window of *product* is given as a table of the widgets it contains, in the order in which they are laid out, with the columns **Widget**, **Class**, **Parent**, **Purpose** and **Realises**.

- **Widget** is the name the widget carries in the design and in the code.
- **Class** is the widget's class — a class of *product* (§4) or a class of Qt.
- **Parent** is the widget that contains it, empty for the window itself.
- **Realises** names the requirement IDs the widget realises, or is empty where the widget only groups others.

| Widget | Class | Parent | Purpose | Realises |
| --- | --- | --- | --- | --- |
| `MainWindow` | `GwbMainWindow` | | the main window | FR-001 |
| `groupBoxCInp` | `QGroupBox` | `MainWindow` | the group of the compiler input file | FR-001 |
| `lineEditCInp` | `Gc3LineEdit` | `groupBoxCInp` | the path of the compiler input file | FR-007, FR-012 |

A widget that a designer tool generates into a '.ui' file appears in this table as well: the table is the design, the '.ui' file is one rendering of it.

### 7.2 Layout sketch

Every window is given as a sketch of where its groups and widgets sit, in a fenced block of fixed-width characters, so that the arrangement is versioned as text alongside the widget tree:

```
+-- MainWindow ------------------------------------------------+
| [Help]                                                       |
| +-- groupBoxCCInp -----------+ +-- groupBoxCOut -----------+ |
| | [lineEditCCInp     ] [...] | | (<) page 1/3 (>)  [detach]| |
| | [labelIndicatorCCInp]      | | +-----------------------+ | |
| | +------------------------+ | | | plainTextEditStdOut   | | |
| | | plainTextEditCCInp     | | | +-----------------------+ | |
| | +------------------------+ | +---------------------------+ |
| +----------------------------+                               |
+--------------------------------------------------------------+
```

The sketch states the arrangement and the nesting, not the pixel sizes: a widget's exact geometry is settled where the window is built, within the arrangement the sketch fixes.

## 8. Data flow

The processing pipeline of *product* is documented as an ordered sequence of stages (e.g. stdin → lexer → incremental parser state → output), naming the type that flows between each stage. Each stage references the component (§3) that implements it.

## 9. Error handling strategy

- All expected failure is represented in types — `std::optional`, `std::expected`, or a dedicated result type — never by an out-of-range access, a null dereference, or an uncaught exception.
- Where the design uses exceptions, it states which component throws, which catches, and which requirement describes that failure behaviour. An exception that crosses a component boundary is part of that component's public interface and is declared with it (§5).
- An error type's enumerators or members are declared per §4 and traced to the requirement(s) describing the corresponding failure behaviour.

## 10. Side-effect boundaries

- Parsing and data-transformation logic is free of side effects: it reads its input through its parameters and returns its result, touching no file, no environment variable, no global or static mutable state. Side effects are confined to the components that the design names as effectful, and to the application's entry point.
- For every function declared per §5, the design states whether it is free of side effects or performs I/O, either by grouping functions under a "Free of side effects" / "Effectful" subheading or by an inline note.
- A member function that changes no observable state of its object is declared `const` (§5). `const` is the marker in the declaration; the grouping or note says what the function does beyond its object.

## 11. Traceability

- Every design element (component, type, or function declaration) that realises one or more requirements states those requirement IDs directly at its declaration (§4, §5), keeping requirement → design traceability explicit at the point of use.
- Before the agent asks gate question G ('01_01_management.md' § Change cycle → Gate), every requirement categorised as Functional or Interface ('02_01_requirements.md' §4.1) is realised by at least one design element.

## 12. Template

````
# Design

Product name: *<name>*

This design follows the structure defined in '02_02_design.md'.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | 01_01_management.md | Management requirements |
| [2] | 02_01_requirements.md | Requirements guideline |
| [3] | 02_02_design.md | Design guideline |
| [4] | <requirements specification file name> | Specification |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | (see Requirements specification, § Terms — not redefined here) |
| *<name>* | <context (optional)> | <definition> |

## Scope

Covers: ...
Out of scope: ...

## Architecture overview

| Component | Files | Responsibility | Depends on |
| --- | --- | --- | --- |
| `genc3::<name>` | `<name>.h`, `<name>.cpp` | ... | ... |

## Type declarations

```cpp
// realises FR-...
struct <Name>
{
    ...
};
```

## Function signatures

```cpp
// realises FR-...
<ReturnType> <name>(<parameters>);
```

## Class diagram

```mermaid
classDiagram
    class <Name> {
        +<type> <attribute>
        +<ReturnType> <operation>()
    }
    <Name> --> <Other> : <relationship>
```

## User interface design

### <Window name>

| Widget | Class | Parent | Purpose | Realises |
| --- | --- | --- | --- | --- |
| `<name>` | `<Class>` | `<parent>` | ... | FR-... |

```
+-- <Window name> ---------------+
| ...                            |
+--------------------------------+
```

## Data flow

input → ... → output

## Error handling strategy

...

## Side-effect boundaries

Free of side effects: ...
Effectful: ...

## Traceability

...

## Open questions

...
````
