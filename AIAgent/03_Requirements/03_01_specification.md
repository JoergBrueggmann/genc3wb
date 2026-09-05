# Specification

Product name: *genc³*

This specification follows the structure defined in '02_01_requirements.md'.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [AD01] | 01_01_management.md | Management requirements |
| [AD02] | 02_01_requirements.md | Requirements guideline |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | A universal meta compiler-compiler build system, named *genc³*. |
| *symbol tree* | | see FR-002 |
| *parser result* | | see FR-013 |
| *parsing error* | | see FR-014 |
| *terminal char* | | A single literal character, matched literally against the input. |
| *sequence* | | A composition of none, one, or more syntax elements, matched in order, one after another. |
| *selection* | | A composition of none, one, or more alternative syntax elements, of which the first that matches is taken. |
| *empty element* | | The syntax element that matches nothing. |
| *memoisation* | | see FR-005 |
| *read position* | | see FR-016 |
| *line break* | | A single line feed character (U+000A). |
| *stdin* | CLI | The standard input stream, as defined by POSIX (IEEE Std 1003.1). |
| *stdout* | CLI | The standard output stream, as defined by POSIX (IEEE Std 1003.1). |
| *stderr* | CLI | The standard error stream, as defined by POSIX (IEEE Std 1003.1). |
| *binary form* | | A serialisation of the *symbol tree* not meant to be read by a human; see FR-030, IR-009. |
| *EBNF form* | | A rendering of the configured syntax as EBNF rules, meant to be read by a human; see FR-046. |
| *error tree* | | Where the input symbol stream does not match: what did not match, and where; see FR-014, FR-031 to FR-035. |
| *human-readable switch* | CLI | The program argument selecting the human-readable form for the *symbol tree* file; see IR-008. |
| *lookahead* | | The number of characters beyond a symbol's matched input that were examined to derive it; see FR-036. |
| *reused derivation* | | see FR-037 |
| *reuse ground* | | see FR-039 |
| *memo switch* | CLI | The program argument selecting the display of the *reused derivations*; see IR-012. |
| *detail keyword* | CLI | The program argument selecting how much detail is rendered per *reused derivation*; see IR-013. |
| *EBNF switch* | CLI | The program argument selecting the rendering of the configured syntax in *EBNF form*; see IR-018. |
| *configured generator* | | The generator the *product* composes text with, configured in a *generator section* separately from the configured syntax; see FR-050, FR-093, C-004. |
| *configuration file* | | The one file stating the configured syntax and every *configured generator* of a run; see FR-096, IR-027. |
| *syntax section* | | The section of the *configuration file* stating the configured syntax; see FR-097, C-002. |
| *generator section* | | A section of the *configuration file* stating one *configured generator*; see FR-097, FR-098, C-004. |
| *value clause* | | The clause of a *generator section* stating the *value* an alternative's *generator element* yields; see FR-101. |
| *text clause* | | The clause of a *generator section* stating the text an alternative's *generator element* emits; see FR-102. |
| *positional reference* | | The denotation, in a clause, of one item of the alternative the clause belongs to; see FR-107. |
| *seed syntax* | | The configured syntax the *product* carries, by which a *configuration file* is parsed; see FR-111. |
| *generation result* | | see FR-051 |
| *generator element* | | see FR-055 |
| *literal* | | The *generator element* emitting the text it carries; see FR-058. |
| *flattening* | | The *generator element* emitting the input symbol stream its *focus*'s subtree was derived from; see FR-059. |
| *concatenation* | | The *generator element* emitting the texts of its *generator elements*, in order; see FR-060. |
| *branch* | | The *generator element* taking one of its *generator elements* by the index its *focus* carries as a *selection* symbol; see FR-061. |
| *choice* | | The *generator element* taking the first of its alternatives whose guard holds; see FR-063. |
| *reference* | | The *generator element* citing a *generator rule* by name, with arguments, instead of containing it; see FR-064. |
| *dispatch* | | The *generator element* citing the *generator rule* named after its *focus*; see FR-066. |
| *traversal* | | The *generator element* applied at each symbol a *path* selects; see FR-067. |
| *binding* | | The *generator element* binding a name to a *value* for a nested *generator element*; see FR-070. |
| *yield* | | The *generator element* emitting a nested *generator element*'s text while yielding the *value* of a *calculation*; see FR-071. |
| *focus* | | The symbol of the *symbol tree* a *generator element* is applied to; see FR-053. |
| *emission* | | see FR-054 |
| *generator rule* | | see FR-073 |
| *generator definitions table* | | The mapping from names to the *generator rules* of the *configured generator*, against which a *reference* and a *dispatch* are resolved; see FR-074. |
| *path* | | see FR-076 |
| *value* | | see FR-078 |
| *calculation* | | see FR-081 |
| *named operation* | | An operation a *calculation* may apply, given to the *product* by name rather than fixed by it; see FR-084. |
| *generation error* | | Where the *configured generator* does not apply: what failed, and at which symbol; see FR-088 to FR-092. |
| *configured failure* | | The *generator element* by which a *configured generator* states that it does not apply; see FR-072. |
| *generator switch* | CLI | The program argument selecting the generation of text from the *symbol tree*; see IR-021. |

## Scope

In scope: incrementally parsing an input symbol stream, according to a grammar configured in the *configuration file* (see FR-003, FR-096 to FR-099, FR-111, FR-112, C-002), inserting it at a specified position into an existing (serialised) *symbol tree*, producing a *parser result*, writing that result to output (see IR-004 to IR-006), making visible which derivations of the existing *symbol tree* the parse reused (see FR-037 to FR-045, IR-012 to IR-017), rendering the configured syntax itself in *EBNF form* (see FR-046 to FR-049, IR-018 to IR-020), and composing text from the *symbol tree* according to a generator configured in that same *configuration file* (see FR-050 to FR-110, IR-021 to IR-029, C-004).

Out of scope: reporting the difference between the *symbol tree* given as input and the resulting *symbol tree*; configuring the syntax or a generator in the *product*'s own source code (see C-002, C-004).

## Functional requirements

### Parsing and symbol derivation

FR-001 [Functional]: The *product* shall parse the input symbol stream and, at the specified position, overwrite the specified width of symbols in the existing *symbol tree*, producing a *parser result*.

FR-002 [Data]: The *symbol tree* shall represent a symbol as a structure with child symbols, recursively, or be empty.

FR-003 [Data]: The *product* shall compose syntax from *terminal chars*, *sequences*, *selections*, and *empty elements*.

FR-004 [Behavioural]: When the configured syntax is left-recursive, the *product* shall terminate.

FR-005 [Behavioural]: When a parsing trial repeats a reading position and syntax element already recorded, the *product* shall reuse the recorded outcome instead of re-attempting the trial.

FR-006 [Data]: Each symbol in the *symbol tree* shall identify one syntax element.

FR-007 [Behavioural]: When a syntax element matches the input, the *product* shall derive a symbol that identifies that syntax element.

FR-008 [Behavioural]: When a *terminal char* matches the input, the derived symbol shall be a *terminal char* symbol with no child symbols.

FR-009 [Behavioural]: When a *sequence* matches the input, the derived symbol shall be a *sequence* symbol with one child symbol per matched syntax element of the *sequence*, in the order matched.

FR-010 [Behavioural]: When a *selection* matches the input, the derived symbol shall be a *selection* symbol with one child symbol, derived from the first alternative syntax element that matched.

FR-011 [Behavioural]: When an *empty element* matches the input, the derived symbol shall be an *empty element* symbol with no child symbols.

FR-012 [Behavioural]: When a *selection* has no alternative syntax elements, the *selection* shall not match the input.

### Parser result, read position, and naming

FR-013 [Data]: The *parser result* shall be either a *parsing error* or a *symbol tree*.

FR-014 [Data]: A *parsing error* shall carry an *error tree*.

FR-015 [Data]: Each symbol in the *symbol tree* shall have the *read position* at which its derivation began.

FR-016 [Data]: A *read position* shall be composed of a line number and a column number, each counted from 1.

FR-017 [Data]: A *sequence*, *selection*, or *empty element* may carry a name, being the name of the rule of the *syntax section* that states it (see C-002).

### Human-readable and binary forms of the symbol tree

FR-018 [Data]: In human-readable form, the *symbol tree* shall be rendered as one line of text per symbol, in depth-first order, each symbol immediately followed by its child symbols.

FR-019 [Data]: A rendered symbol line shall consist of the symbol's element kind, the index carried by a *selection* symbol (see FR-029) enclosed in square brackets, the element's identifier enclosed in parentheses where the element has an identifier, a colon, the *read position*'s line number, a colon, the *read position*'s column number, and, for a *sequence* or *selection* symbol, a colon followed by the symbol's *lookahead* (see FR-036).

FR-020 [Data]: The element kind shall be rendered as `Char` for a *terminal char*, `Seq` for a *sequence*, `Sel` for a *selection*, and `Empty` for an *empty element*.

FR-021 [Data]: The element identifier shall be rendered as the matched character enclosed in single quotes for a *terminal char*, and as the element's name otherwise.

FR-022 [Data]: The root symbol's rendered line shall have no prefix.

FR-023 [Data]: A *sequence*, *selection*, or *empty element* without a name shall have no identifier.

FR-024 [Data]: A *read position*'s line number shall be 1 plus the number of *line breaks* in the input preceding that *read position*.

FR-025 [Data]: When at least one *line break* precedes a *read position*, that *read position*'s column number shall be 1 plus the number of characters between the last such *line break* and that *read position*.

FR-026 [Data]: When no *line break* precedes a *read position*, that *read position*'s column number shall be 1 plus the number of characters preceding that *read position*.

FR-027 [Data]: A non-root symbol's rendered line shall be prefixed with one guide per ancestor symbol beyond the root, in order from the outermost such ancestor to the innermost, followed by `+-`.

FR-028 [Data]: A guide shall be a vertical bar followed by a space where its ancestor symbol has a following sibling symbol, and two spaces otherwise.

FR-029 [Data]: A *selection* symbol shall carry the index, counted from 0, of the alternative syntax element from which its child symbol was derived.

FR-030 [Data]: A *symbol tree* serialised in *binary form* shall be readable back as the same *symbol tree*.

### Error tree and lookahead

FR-031 [Data]: Each error in the *error tree* shall identify one syntax element and have the *read position* at which that element's matching began.

FR-032 [Behavioural]: When a *sequence* does not match, its error shall have one child error: the error of its first non-matching element.

FR-033 [Behavioural]: When a *selection* does not match, its error shall have no child error.

FR-034 [Data]: When the input symbol stream extends beyond the root syntax element's match, the *error tree* shall consist of one error, without a syntax element, at the *read position* of the first unconsumed character.

FR-035 [Data]: In human-readable form, the *error tree* shall be rendered as one line of text per error, each error immediately followed by its child error, each line consisting of the element kind (FR-020, and `Ref` for a *reference*, or `Unconsumed` for the error of FR-034), the element's identifier enclosed in parentheses where the element has one (FR-021, FR-023), a colon, the *read position*'s line number, a colon, and the *read position*'s column number, a child error's line prefixed per FR-027 and FR-028.

FR-036 [Data]: Each *sequence* and *selection* symbol shall carry its *lookahead*: the number of characters beyond the symbol's matched input that were examined to derive it, such that deriving the symbol read no character at or beyond the position after its matched input plus its *lookahead*. A *terminal char* symbol's derivation examines exactly its own character, and an *empty element* symbol's derivation examines nothing, so neither carries a *lookahead*.

### Reused derivations

A position in this subsection is a position in the overwritten input symbol stream, counted from `0` as in IR-002, rather than a *read position*.

FR-037 [Data]: A *reused derivation* shall be the derivation of a symbol of the existing *symbol tree* that the *product* takes over, rather than deriving that symbol again, for the parse of the overwritten input symbol stream (see FR-001, FR-005).

FR-038 [Data]: Each *reused derivation* shall carry the position at which it applies in the overwritten input symbol stream, the syntax element its symbol was derived from, the position after its symbol's matched input, and its *reuse ground*.

FR-039 [Data]: A *reuse ground* shall be either that the *reused derivation* applies at or beyond the end of the overwritten region, carrying the number of positions by which the overwrite shifts it, or that the *reused derivation* applies before the insertion position, carrying the position after its symbol's matched input plus its symbol's *lookahead*, that position being at most the insertion position.

FR-040 [Data]: In human-readable form, the *reused derivations* shall be rendered as one line of text per *reused derivation*, in ascending order of the position at which each applies, and, where several apply at the same position, in the depth-first order of their symbols in the existing *symbol tree*.

FR-041 [Data]: The rendered *reused derivations* shall be preceded by one line of text reading `Reused derivations`.

FR-042 [Data]: A rendered *reused derivation* line shall consist of the position at which it applies, a colon, the element kind (see FR-020), the element's identifier enclosed in parentheses where the element has one (see FR-021, FR-023), a colon, and the position after its symbol's matched input.

FR-043 [Data]: When the *detail keyword* is `lookahead` or `ground`, a rendered *reused derivation* line shall additionally carry, after the position after its symbol's matched input, a colon and its symbol's *lookahead* (see FR-036).

FR-044 [Data]: When the *detail keyword* is `ground` and the *reused derivation*'s *reuse ground* is that it applies at or beyond the end of the overwritten region, the rendered line shall additionally carry a colon, the word `shifted`, a space, and the number of positions by which the overwrite shifts it.

FR-045 [Data]: When the *detail keyword* is `ground` and the *reused derivation*'s *reuse ground* is that it applies before the insertion position, the rendered line shall additionally carry a colon, the word `bounded`, a space, and the position after its symbol's matched input plus its symbol's *lookahead*.

### EBNF form of the configured syntax

FR-046 [Data]: The *product* shall render the configured syntax in *EBNF form*, as one rule per named syntax element of the configured syntax, one rule per line of text, the root syntax element's rule first and every further rule in ascending order of its syntax element's name.

FR-047 [Data]: A rendered EBNF rule shall consist of the syntax element's name, a space, an equals sign, a space, that syntax element's rendering, and a semicolon.

FR-048 [Data]: In *EBNF form*, a syntax element carrying a name, other than the element whose rule is being rendered, shall be rendered as that name; a *terminal char* as its character enclosed in single quotes; an *empty element* as a left round bracket immediately followed by a right round bracket; a *reference* as the name it cites; a *sequence* as its elements' renderings separated by a comma and a space; a *selection* as its alternatives' renderings separated by a space, a vertical bar, and a space; and a *sequence* or *selection* that is not the element whose rule is being rendered, enclosed in a left round bracket and a space, and a space and a right round bracket.


FR-049 [Data]: A configured syntax rendered in *EBNF form* shall be readable back as a configured syntax deriving, from every input symbol stream, the same *symbol tree* as the rendered one.

### Text generation

FR-050 [Functional]: The *product* shall compose text from the *symbol tree*, according to a *configured generator*, producing a *generation result*.

FR-051 [Data]: A *generation result* shall be either the generated text or a *generation error*.

FR-052 [Data]: The *product* shall compose a *configured generator* from *generator elements*.

FR-053 [Behavioural]: The *product* shall apply a *generator element* to one symbol of the *symbol tree*, that symbol being the *generator element*'s *focus*.

FR-054 [Data]: An *emission* shall be composed of the text a *generator element* emits at its *focus* and the *value* it yields there.

FR-055 [Data]: A *generator element* shall be a *literal*, a *flattening*, a *concatenation*, a *branch*, a *choice*, a *reference*, a *dispatch*, a *traversal*, a *calculation*, a *binding*, a *yield*, or a *configured failure*.

FR-056 [Behavioural]: When the *product* applies a *generator element* to a *focus* under bound names it has already applied that *generator element* to under those same bound names, it shall reuse the *emission* recorded for that application instead of producing it again.

FR-057 [Behavioural]: When applying a *generator element* to a *focus* under bound names re-enters that same *generator element* at that same *focus* under those same bound names, the *product* shall produce a *generation error*.

### Generator elements

FR-058 [Behavioural]: A *literal* shall emit the text it carries and shall yield no *value*.

FR-059 [Behavioural]: A *flattening* shall emit the input symbol stream from which its *focus*'s subtree was derived, and shall yield no *value*.

FR-060 [Behavioural]: A *concatenation* shall emit, in order, the texts its *generator elements* emit at its *focus*, and shall yield no *value*.

FR-061 [Behavioural]: When its *focus* is a *selection* symbol and it has a *generator element* at the index that symbol carries (see FR-029), a *branch* shall emit and yield the *emission* of that *generator element*.

FR-062 [Behavioural]: When its *focus* is not a *selection* symbol, or it has no *generator element* at the index that symbol carries, a *branch* shall produce a *generation error*.

FR-063 [Behavioural]: A *choice* shall emit and yield the *emission* of the *generator element* of its first alternative whose guarding *calculation* yields a true truth value; where no alternative's guard yields one, it shall produce a *generation error*.

FR-064 [Behavioural]: A *reference* shall emit and yield the *emission* of the *generator rule* it cites, applied to its *focus*, the *values* of its arguments bound to that *generator rule*'s parameter names and no other name bound.

FR-065 [Behavioural]: When a *reference* cites a name the *generator definitions table* does not hold, or carries a number of arguments other than the cited *generator rule*'s number of parameter names, it shall produce a *generation error*.

FR-066 [Behavioural]: When its *focus* carries a name the *generator definitions table* holds a *generator rule* of, a *dispatch* shall emit and yield the *emission* of that *generator rule*, applied to its *focus* with no name bound; otherwise it shall emit and yield the *emission* of a *flattening*.

FR-067 [Behavioural]: A *traversal* shall emit, in order, the texts its *generator element* emits at each symbol its *path* selects from its *focus*, and shall yield the *value* that *generator element* yields where the *path* selects exactly one symbol, and no *value* otherwise.

FR-068 [Behavioural]: When a *traversal*'s *path* selects no symbol from its *focus*, the *traversal* shall produce a *generation error*.

FR-069 [Behavioural]: A *calculation* used as a *generator element* shall emit the text its *value* renders as (see FR-080) and shall yield that *value*.

FR-070 [Behavioural]: A *binding* shall emit and yield the *emission* of its *generator element*, applied to its *focus* with the *value* of its *calculation* bound to its name in addition to the names already bound.

FR-071 [Behavioural]: A *yield* shall emit the text its *generator element* emits at its *focus* and shall yield the *value* of its *calculation*.

FR-072 [Behavioural]: A *configured failure* shall produce a *generation error* carrying the text it states.

FR-073 [Data]: A *generator rule* shall be composed of a sequence of parameter names and a *generator element*.

FR-074 [Data]: The *generator definitions table* shall map names to the *generator rules* of the *configured generator*, and shall be the mapping against which a *reference* and a *dispatch* resolve a name.

FR-075 [Data]: A *generator element* shall cite a *generator rule* by name, through a *reference* or a *dispatch*, rather than contain it, so that every *generator element* is finite.

### Paths

FR-076 [Data]: A *path* shall select, from a *focus*, either that *focus*, or its child symbol at a given index, or all its child symbols in order, or its parent symbol, or the root symbol of the *symbol tree*, or every nearest descendant symbol carrying a given name, or the symbols a second *path* selects from each symbol a first *path* selects.

FR-077 [Data]: A *path* selecting every nearest descendant symbol carrying a given name shall not select a symbol having, below the *focus*, an ancestor carrying that name.

### Values and calculations

FR-078 [Data]: A *value* shall be no value, a number, text, a truth value, or a sequence of *values*.

FR-079 [Data]: A number *value* shall be a rational number of unbounded magnitude.

FR-080 [Data]: A *value* shall render as the empty text where it is no value; as itself where it is text; as `true` or `false` where it is a truth value; as its digits, preceded by a minus sign where it is negative, where it is a whole number; as its numerator, a solidus, and its denominator, in lowest terms and preceded by a minus sign where it is negative, where it is a number that is not whole; and as its *values*' renderings in order, one after another, where it is a sequence of *values*.

FR-081 [Data]: A *calculation* shall be a constant *value*, a bound name, the *value* a *generator element* yields at the *focus*, the text a *generator element* emits at the *focus*, the index the *focus* carries as a *selection* symbol, a unary operation applied to a *calculation*, a binary operation applied to two *calculations*, a conditional of three *calculations*, or a *named operation* applied to a sequence of *calculations*.

FR-082 [Data]: A unary operation shall be arithmetic negation, logical negation, the number a text denotes, the text a *value* renders as, or the number of characters of a text.

FR-083 [Data]: A binary operation shall be addition, subtraction, multiplication, division, exponentiation, concatenation of text, equality, inequality, one of the four orderings, conjunction, or disjunction.

FR-084 [Data]: The *named operations* a *calculation* may apply shall be given to the *product* as a mapping from names to operations, so that a *named operation* is added without a change to the *product*'s source.

FR-085 [Behavioural]: When a *calculation* applies an operation to a *value* whose kind that operation is not defined for, the *product* shall produce a *generation error*.

FR-086 [Behavioural]: When a *calculation* divides by zero, or raises a number to an exponent that is not whole, the *product* shall produce a *generation error*.

FR-087 [Behavioural]: When a *calculation* names a name that is not bound, the *product* shall produce a *generation error*.

### Generation error

FR-088 [Data]: Each error in the *generation error* shall identify what failed, identify the symbol at which it failed, and have the *read position* at which that symbol's derivation began.

FR-089 [Data]: An error shall identify the symbol at which it failed by that symbol's number in depth-first order, counted from 0, which is the number of the line that symbol occupies in the human-readable form of the *symbol tree* (see FR-018), counted from 0.

FR-090 [Data]: When a failure arises within a *generator rule* cited by a *reference* or a *dispatch*, the error of that citation shall have one child error: the error the failure arose from.

FR-091 [Data]: In human-readable form, the *generation error* shall be rendered as one line of text per error, each error immediately followed by its child error, a child error's line prefixed per FR-027 and FR-028.

FR-092 [Data]: A rendered *generation error* line shall consist of what failed, the name it concerns enclosed in parentheses where it concerns one, a colon, the *read position*'s line number, a colon, the *read position*'s column number, a colon, and the symbol's number per FR-089.

### Configuration of syntax and generator

FR-093 [Data]: A *configured generator* shall be configured in a *generator section* of its own, separately from the *syntax section*, such that several *configured generators* are configured against one configured syntax.

FR-094 [Data]: Each *configured generator* shall carry a name, by which the *configured generator* to compose with is selected.

FR-095 [Behavioural]: When a *configured generator* holds a *generator rule* under a name that neither names a syntax element of the configured syntax nor is cited by a *reference* of that *configured generator*, the *product* shall compose no text with that *configured generator*.

FR-096 [Data]: The *configuration file* shall state the configured syntax and every *configured generator* of a run.

FR-097 [Data]: The *configuration file* shall be composed of one *syntax section*, stating the configured syntax, and one or more *generator sections*, each stating one *configured generator*.

FR-098 [Data]: Each *generator section* shall carry the name of the *configured generator* it states (see FR-094).

FR-099 [Data]: The *syntax section* shall state the configured syntax in *EBNF form* (see FR-046 to FR-049).

### Generator sections

FR-100 [Data]: A *generator section* shall state, for each alternative of each rule of the *syntax section*, at most one *value clause* and at most one *text clause*.

FR-101 [Data]: A *value clause* shall state a *calculation*, and the *generator element* of the alternative it belongs to shall yield that *calculation*'s *value*.

FR-102 [Data]: A *text clause* shall state a *calculation*, and the *generator element* of the alternative it belongs to shall emit the text that *calculation*'s *value* renders as (see FR-080).

FR-103 [Behavioural]: When an alternative states a *value clause* and no *text clause*, its *generator element* shall emit the text the *value clause*'s *value* renders as.

FR-104 [Behavioural]: When an alternative states a *text clause* and no *value clause*, its *generator element* shall yield no *value*.

FR-105 [Behavioural]: When an alternative states neither a *value clause* nor a *text clause*, its *generator element* shall be a *flattening*.

FR-106 [Data]: A *generator section* may state a *value clause* or a *text clause* for a rule rather than for one of its alternatives; such a clause shall apply to every alternative of that rule stating no clause of that kind.

### Positional references

FR-107 [Data]: A *positional reference* shall denote one item of the alternative the clause it appears in belongs to, the items counted from 1.

FR-108 [Data]: A *positional reference* shall be numbered at least 1 and at most the number of items of that alternative.

FR-109 [Data]: A *positional reference*'s *value* shall be the *value* a *dispatch* yields at the symbol its *path* selects.

FR-110 [Data]: A *positional reference*'s *path* shall be: where its rule has one alternative, the child symbol at the index one less than the reference's number; where its rule has several alternatives and its alternative has one item, the child symbol at index 0; and where its rule has several alternatives and its alternative has several items, the child symbol at index 0 and, from that symbol, the child symbol at the index one less than the reference's number.

### Bootstrap

FR-111 [Data]: The *product* shall carry a *seed syntax*, being the configured syntax by which a *configuration file* is parsed.

FR-112 [Data]: A *configuration file* whose *syntax section* is the *seed syntax* rendered in *EBNF form* shall parse every *configuration file* as the *seed syntax* does.

## Non-functional requirements

NFR-001 [Performance]: The *product* shall parse an input symbol stream of 100000 characters within 2 seconds.

NFR-002 [Performance]: When the overwrite replaces at most 1 symbol at the beginning of an existing *symbol tree* of 100000 symbols, the *product* shall parse within 1 second.

NFR-003 [Performance]: When the overwrite replaces at most 1 symbol in the middle of an existing *symbol tree* of 100000 symbols, the *product* shall parse within 1 second.

NFR-004 [Performance]: When the overwrite replaces at most 1 symbol at the end of an existing *symbol tree* of 100000 symbols, the *product* shall parse within 1 second.

NFR-005 [Performance]: The *product* shall compose text from a *symbol tree* of 100000 symbols within 2 seconds.

## Interface requirements

### CLI

IR-001 (input): The *product* shall accept an input symbol stream to be parsed from *stdin*.

IR-002 (input): The *product* shall accept the insertion position and the width to overwrite, both as decimal numbers, as program arguments following the switch `--pos` (or `-p`); the beginning position is `0`.

Example: *--pos 3 4* means that, after the third symbol, four symbols are overwritten.

IR-003 (input, output): The *product* shall accept a serialised *symbol tree* to be extended, as a file path given as a program argument following the switch `--symboltree` (or `-s`).

IR-004 (output): When the *parser result* is a *symbol tree*, the *product* shall write that *symbol tree* to the file given by `--symboltree`/`-s`, creating that file where it does not exist and replacing its content otherwise; when the *parser result* is a *parsing error*, the file is left unchanged.

IR-005 (output): When the *parser result* is a *symbol tree*, the *product* shall write it, in human-readable form (see FR-018 to FR-029), to *stdout*.

IR-006 (output): When the *parser result* is a *parsing error*, the *product* shall write it, in human-readable form, to *stderr*.

IR-007 (input): When the file given by `--symboltree`/`-s` does not exist, the *product* shall use the empty *symbol tree* as the existing *symbol tree*.

IR-008 (input): The *product* shall accept the *human-readable switch*, `--human-readable-symbol-tree` (or `-hr`), as a program argument.

IR-009 (output): When the *human-readable switch* is absent, the *product* shall write the *symbol tree* file in *binary form*.

IR-010 (output): When the *human-readable switch* is present, the *product* shall write the *symbol tree* file in human-readable form (FR-018 to FR-029).

IR-011 (input): The *product* shall read the *symbol tree* file in the form selected by the *human-readable switch*.

IR-012 (input): The *product* shall accept the *memo switch*, `--show-memo` (or `-sm`), as a program argument.

IR-013 (input): The *product* shall accept the *detail keyword*, being `basic`, `lookahead`, or `ground`, as a program argument following the *memo switch*.

IR-014 (input): When the *memo switch* is present without a following *detail keyword*, the *product* shall use `basic` as the *detail keyword*.

IR-015 (output): When the *memo switch* is present and the *parser result* is a *symbol tree*, the *product* shall write the *reused derivations*, in human-readable form (see FR-040 to FR-045), to *stdout*, following the *symbol tree* written per IR-005.

IR-016 (output): When the *memo switch* is absent, the *product* shall write no *reused derivation*.

IR-017 (output): When the *parser result* is a *parsing error*, the *product* shall write no *reused derivation*.

IR-018 (input): The *product* shall accept the *EBNF switch*, `--ebnf`, as a program argument.

IR-019 (output): When the *EBNF switch* is present, the *product* shall write the configured syntax in *EBNF form* (see FR-046 to FR-048) to *stdout*.

IR-020 (output): When the *EBNF switch* is present, the *product* shall parse no input symbol stream and shall write no *symbol tree*.

IR-021 (input): The *product* shall accept the *generator switch*, `--generate` (or `-g`), as a program argument.

IR-022 (output): When the *generator switch* is present and the *generation result* is the generated text, the *product* shall write that text to *stdout*, following the *symbol tree* written per IR-005 and any *reused derivations* written per IR-015.

IR-023 (output): When the *generator switch* is present and the *generation result* is a *generation error*, the *product* shall write that *generation error*, in human-readable form (see FR-091, FR-092), to *stderr*.

IR-024 (output): When the *generator switch* is absent, or the *parser result* is a *parsing error*, the *product* shall write no generated text and no *generation error*.

IR-025 (input): The *product* shall accept the name of the *configured generator* to compose with, as a program argument following the *generator switch*.

IR-026 (output): When a *configured generator* holds a *generator rule* under a name that neither names a syntax element of the configured syntax nor is cited by a *reference* of that *configured generator*, the *product* shall write each such name to *stderr*.

IR-027 (input): The *product* shall accept the *configuration file*, as a file path given as a program argument following the switch `--config` (or `-c`).

IR-028 (output): When the *configuration file* does not exist, or its content is not a well-formed *configuration file* (see FR-097 to FR-110), the *product* shall write a message naming the fault to *stderr*, and shall neither parse nor compose.

IR-029 (output): When the *EBNF switch* is present and no *configuration file* is given, the *product* shall render the *seed syntax*.

## Constraints

C-001: The *product* shall be implemented using the Haskell Stack toolchain.

C-002: The *product*'s grammar shall be configured in the *syntax section* of the *configuration file*, rather than in the *product*'s source code or via a CLI flag.

C-003: The *product* shall be delivered as a single executable, requiring at run time neither the toolchain of C-001 nor any other build or interpretation step.

C-004: The *product*'s generators shall be configured in the *generator sections* of the same *configuration file* as its grammar (see C-002), rather than in the *product*'s source code or via a CLI flag.

## Open questions

- none -
