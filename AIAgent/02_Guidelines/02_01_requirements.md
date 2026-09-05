# Requirements guideline

This guideline defines the mandatory structure for every requirements specification document produced in phase 3 *Requirements Definition* (folder '03_Requirements') for *product*, in addition to the general rules of '01_01_management.md' (§ Document rules).

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | 01_01_management.md | Management requirements |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | The software product being developed; formally defined by the Requirements specification (folder '03_Requirements'). |
| *functional requirement* | ID prefix `FR-`, recorded in the Functional requirements section | A requirement concerning a result or behavior that shall be provided by a function of the system (Pohl & Rupp). |
| *quality requirement* | ID prefix `NFR-`, recorded in the Non-functional requirements section | A requirement that pertains to a quality concern that is not covered by *functional requirements* (Pohl & Rupp). |
| *constraint* | Recorded in the Constraints section | A requirement that limits the solution space beyond what is necessary for meeting the given *functional requirements* and *quality requirements* (Pohl & Rupp). |
| *non-functional requirement* | | Per Pohl and Rupp, denotes a *quality requirement* or a *constraint* jointly. |
| *interface requirement* | ID prefix `IR-`, recorded in the Interface requirements section | A cross-cutting grouping, by interface, of requirements that are additionally one of the three Pohl/Rupp types (§4.1) depending on what they state. |
| *data perspective* | Applies to *functional requirements* only (§4.2) | The static-structural aspects of the data used, exchanged, or produced by *product*, and its relationships to the system context. |
| *functional perspective* | Applies to *functional requirements* only (§4.2) | How *product* processes data from the system context: which data it manipulates, and which data it transmits back to the system context. |
| *behavioural perspective* | Applies to *functional requirements* only (§4.2) | The states of *product*, the events and conditions that trigger state changes, and the effects of those changes on the system context. |

## 1. Purpose and scope

This guideline applies to all requirements specification documents for *product*. It does not itself contain requirements; it defines how requirements documents must be structured and worded.

## 2. Mandatory document structure

Every requirements specification document must contain the following sections, in this order. A section that has no content must still be present, with its body set to "- none -" — sections are never omitted.

1. **Header** — product name and a one-line reference to this guideline.
2. **Applicable documents** — as required by '01_01_management.md' (§ Document rules → Applicable documents).
3. **Terms** — as required by '01_01_management.md' (§ Document rules → Terms).
4. **Scope** — one short paragraph stating what the document covers, and one stating what is explicitly out of scope.
5. **Functional requirements** — the behavior *product* shall exhibit.
6. **Non-functional requirements** — quality attributes such as performance, reliability, and portability.
7. **Interface requirements** — grouped by interface (e.g. CLI, file format, API); each requirement states the direction of data flow (`input`, `output`, or both when the same channel serves as both, e.g. a file that is read and, on success, overwritten).
8. **Constraints** — technical or organisational constraints (e.g. programming language, platform) that shape but are not themselves behavior.
9. **Open questions** — permitted only while the change cycle is running; must be empty before the agent asks gate question G ('01_01_management.md' § Change cycle → Gate).

A section above — and, recursively, any subsection within it — should be divided into further subsections, each introduced by a heading one level deeper naming the group, once it grows large enough that the grouping aids readability (e.g. beyond about 5 to 10 requirements) and its requirements admit a reasonable grouping (e.g. by feature, module, or interface). Interface requirements groups by interface regardless of count (item 7). A section or subsection short enough to read as one, or whose requirements do not fall into such a grouping, remains flat.

## 3. Requirement identification and wording

- Every individual requirement (functional, non-functional, or interface) is assigned a unique, stable ID: `FR-<n>` for functional, `NFR-<n>` for non-functional, `IR-<n>` for interface — three-digit, zero-padded, incrementing (e.g. `FR-001`).
- IDs are never reused or renumbered, including across later revisions of the document.
- Each requirement is a single, atomic, testable statement using RFC-2119-style wording: "shall" for mandatory, "should" for recommended, "may" for optional. Vague verbs ("supports", "handles", "deals with") are not permitted.
- One requirement per item. A statement joining independent behaviors with "and"/"or" must be split into separate requirements.
- Every requirement's sentence keeps the normative shape defined in §5, and follows the canonical pattern for its type (§4.1) and perspective(s) (§4.2) wherever that pattern can carry its meaning.
- A Term's Definition ('01_01_management.md' § Document rules → Terms) may reference one or more requirement IDs from this section, provided each referenced requirement contributes to the term's meaning — states what the term's referent *is*, or how it behaves — rather than merely mentioning or using the term. A term naming a behaviour is typically defined by a functional requirement of Behavioural perspective (§4.2).

## 4. Requirement categorisation

Every requirement is categorised along two independent dimensions defined in Pohl, K. and Rupp, C., *Requirements Engineering Fundamentals* (the IREB CPRE Foundation Level study guide).

### 4.1 By type

Every requirement is one of the three requirement types:

- *Functional requirement* — "a requirement concerning a result or behavior that shall be provided by a function of the system" — recorded in the Functional requirements section, ID prefix `FR-`.
- *Quality requirement* — "a requirement that pertains to a quality concern that is not covered by functional requirements" — recorded in the Non-functional requirements section, ID prefix `NFR-`.
- *Constraint* — "a requirement that limits the solution space beyond what is necessary for meeting the given functional requirements and quality requirements" — recorded in the Constraints section (§2 item 8).

Per Pohl and Rupp, *non-functional requirement* denotes a *quality requirement* or a *constraint* jointly; within this guideline's document structure the Non-functional requirements section holds *quality requirements* specifically, while *constraints* are kept in their own section so their solution-space-limiting nature stays traceable on its own.

*Interface requirements* (`IR-`) are a cross-cutting grouping by interface, not a fourth Pohl/Rupp category — each *interface requirement* is additionally one of the three types above, depending on what it states. That type is used only to pick the phrasing template (§5); it is not shown as a tag, since the `IR-` prefix already identifies the requirement as an *interface requirement* (see §4.2).

### 4.2 By perspective

The book's Three Perspectives of Requirements apply to *functional requirements* only: they describe what data, what processing, and what states/behaviour a *function* of *product* exhibits. A *quality requirement* or a *constraint* expresses a single quality concern or a solution-space limitation rather than a piece of system function, so it does not decompose into these perspectives and carries no perspective tag.

Every *functional requirement* is tagged with the perspective(s) from which it documents *product*:

- *Data perspective* — the static-structural aspects of the data used, exchanged, or produced by *product*, and its relationships to the system context.
- *Functional perspective* — how *product* processes data from the system context: which data it manipulates, and which data it transmits back to the system context.
- *Behavioural perspective* — the states of *product*, the events and conditions that trigger state changes, and the effects of those changes on the system context.

A *functional requirement* may address more than one perspective. Each applicable perspective is noted next to the requirement's ID, e.g. `FR-001 [Data, Functional]`. An *interface requirement* (`IR-`) carries neither a perspective tag nor a type tag — its `IR-` prefix already identifies it as an *interface requirement*; it is tagged only with its data-flow direction (input/output), per §2 item 7.

### 4.3 By quality category

Every *quality requirement* is tagged with the category of the quality concern it expresses, noted next to its ID in square brackets — e.g. `NFR-001 [Performance]`. The categories follow the product quality characteristics that Pohl and Rupp adopt from ISO/IEC 25010 (functional suitability being excluded, as it is covered by *functional requirements*):

- *Performance* — time behaviour (e.g. response and processing times), resource consumption, and capacity under stated conditions.
- *Compatibility* — co-existence with other products in a shared environment, and interoperability with them.
- *Usability* — how well specified users can learn, operate, and control the product with effectiveness, efficiency, and satisfaction.
- *Reliability* — the degree to which the product performs its functions under stated conditions for a stated period, including availability, fault tolerance, and recoverability.
- *Security* — protection of information and data, so that access is granted only to the degree authorised (confidentiality, integrity, accountability, authenticity).
- *Maintainability* — the effectiveness and efficiency with which the product can be modified, corrected, or adapted, including analysability and testability.
- *Portability* — the effectiveness and efficiency with which the product can be transferred to another hardware, software, or usage environment.

A *quality requirement* addresses exactly one category; a concern spanning two categories is split into one requirement per category, consistent with §3 (one requirement per item).

## 5. Phrasing templates (EBNF)

Every requirement's sentence conforms to the following shape, which is normative:

```
requirement = [ condition ] , subject , modal , predicate , "." ;
```

The productions in §5.1 to §5.5 are the **canonical patterns** for that shape: the form a requirement takes by default. A *functional requirement* (§4.1) selects its canonical pattern by crossing its type with the perspective under which it is being phrased (§4.2) — §5.1 to §5.3 — and a *functional requirement* addressing more than one perspective is composed by phrasing one sentence per applicable perspective. A *quality requirement* or a *constraint* carries no perspective (§4.2) and always uses its single canonical pattern — §5.4 or §5.5, respectively.

A requirement matching its canonical pattern is well-formed by construction. Where a requirement's meaning cannot be carried by that pattern — an intransitive verb, a phrase of means or manner, a construction the pattern's productions do not admit — the requirement still conforms, provided it keeps the normative shape above and satisfies §3: a single, atomic, testable statement in RFC-2119-style wording, free of vague verbs. The canonical patterns are not an exhaustive grammar of requirement sentences, and a requirement is not defective merely for falling outside them.

Notation follows ISO/IEC 14977 EBNF: `=` defines a rule, `,` concatenates, `|` alternates, `[ ]` marks an optional part, `{ }` marks zero-or-more repetition, and quoted text is a literal.

### 5.0 Shared elements

```
subject         = product_subject | entity_subject ;
product_subject = "the" , "*product*" ;
entity_subject  = [ determiner ] , data_entity , [ qualifier ] ;
determiner      = "the" | "a" | "an" | "each" | "every" ;
qualifier       = ? a restrictive phrase narrowing the data entity, e.g. "in the symbol tree" ? ;
modal           = "shall" | "shall not" | "should" | "should not" | "may" ;
predicate       = ? a verb phrase, in the modal's scope, stating what the subject does, what it is, what it contains, or how it is structured or rendered ? ;
condition       = ( "if" | "when" ) , clause , "," ;
comparator      = "at least" | "at most" | "exactly" | "within" ;
value           = number , [ unit ] ;
clause          = ? free-text clause, itself composed of terms per '01_01_management.md' § Document rules → Terms and IDs per §3 ? ;
number          = ? decimal number ? ;
unit            = ? unit of measure ? ;
```

A requirement stating what *product* does takes `product_subject`. A requirement stating what a piece of data is, what it contains, or how it is structured or rendered takes `entity_subject`, naming that data instead. Writing *The product shall produce a symbol tree in which each symbol carries a read position* where *Each symbol in the symbol tree shall carry a read position* is meant adds a level of indirection and makes the requirement harder to test against the data itself. §5.4 already admits both subjects, for the same reason.

`clause`, `number`, `unit`, `qualifier`, `predicate`, and `data_predicate` are not constrained further by this guideline. Every other identifier used inside a canonical pattern (`data_entity`, `process_verb`, `state_name`, `event`, `standard_ref`, `technology_ref`) names one of: a term defined per '01_01_management.md' § Document rules → Terms, written in italics; an ID defined per §3; or an ordinary word or phrase used with its ordinary meaning. The last case follows '01_01_management.md' § Document rules → Terms, which requires a word to be defined as a term only where it is not common knowledge, or where its meaning within *product* must be more specific than its everyday sense — requiring every noun inside a requirement to be a defined term would contradict that rule.

### 5.1 Functional requirement — Data perspective

```
fr_data        = subject , modal , data_predicate , "." ;
data_predicate = ( "accept" | "produce" | "represent" | "provide" ) , data_entity ,
                   [ "with" , "structure" , structure_ref ]
               | ? a verb phrase stating what the subject is, what it contains, or how it is structured or rendered ? ;
```
Examples: *The product shall accept an input stream with structure defined by the CLI grammar.* — *A read position shall be composed of a line number and a column number, each counted from 1.*

The free-form alternative does not relax §3: the sentence must still be a single, atomic, testable statement, and the vague verbs named there ("supports", "handles", "deals with") remain prohibited.

### 5.2 Functional requirement — Functional perspective

```
fr_func = subject , modal , process_verb , object ,
          [ "from" , source ] , [ "into" , target ] , "." ;
```
Example: *The product shall parse stdin into a symbol tree.*

### 5.3 Functional requirement — Behavioural perspective

```
fr_behav = [ condition ] , subject , modal , process_verb , object ,
           [ "," , "transitioning" , "to" , state_name ] , "." ;
```
Example: *When an unrecognised token is read, the product shall report an error, transitioning to state Failed.*

### 5.4 Quality requirement

```
nfr = subject , modal , quality_attribute , [ "of" , object_ref ] ,
      comparator , value , "." ;
```
Examples: *The product shall parse a 1 MB input file within 2 seconds.* — *The parsed symbol tree shall retain at most 10 MB of buffered input.*

### 5.5 Constraint

```
c = subject , modal , ( "use" | "implement" | ( "conform" , "to" ) ) ,
    technology_or_standard_ref , [ "for" , object_ref ] , "." ;
```
Examples: *The product shall be implemented using the Qt framework.* — *Input data shall conform to UTF-8 encoding.*

## 6. Traceability

Requirements must be phrased so each can be referenced by its ID from phase 4 *Design Definition* and phase 5 *Implementation and Test*, keeping the requirement → design → test chain traceable end to end.

## 7. Template

```
# Specification

Product name: *<name>*

This specification follows the structure defined in '02_01_requirements.md'.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | 01_01_management.md | Management requirements |
| [2] | 02_01_requirements.md | Requirements guideline |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | <definition> |
| *<name>* | <context (optional)> | <definition, may reference FR-.../NFR-.../IR-...> |

## Scope

In scope: ...
Out of scope: ...

## Functional requirements

FR-001 [<perspective(s)>]: The *product* shall ...

## Non-functional requirements

NFR-001 [<category>]: The *product* shall ...

## Interface requirements

### <Interface name>

IR-001 (<input|output>): The *product* shall ...

## Constraints

...

## Open questions

...
```
