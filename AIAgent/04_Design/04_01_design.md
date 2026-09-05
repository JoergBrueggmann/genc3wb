# Design

Product name: *genc³*

This design follows the structure defined in '02_02_design.md'.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [AD01] | 01_01_management.md | Management requirements |
| [AD02] | 03_01_specification.md | Specification |
| [AD03] | 02_02_design.md | Design guideline |
| [AD04] | 02_04_coding.md | Coding conventions |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | (see Specification [AD02], § Terms — not redefined here) |
| *syntax element* | `Genc3.Element` | A *terminal char*, *sequence*, *selection*, *empty element*, or *reference*, as a value of type `Syntax`; the configured counterpart of a symbol. |
| *reference* | `Genc3.Element` | A *syntax element* that cites a named *syntax element* by its *name* instead of embedding it; resolved against the *definitions table* at derivation time. |
| *definitions table* | `Genc3.Element` | The mapping from *names* to the named *syntax elements* of the configured syntax, against which a *reference* is resolved. |
| *seed* | `Genc3.Parser` | The *derivation outcome* recorded for a trial in progress: initially no match and, at a *reference*, grown to the fixed point of that *reference*'s derivation (see § Function signatures, `Genc3.Parser`). |
| *indexed stream* | `Genc3.Stream` | The input symbol stream converted once into an array, giving constant-time access to the character at an *offset*; the form the parser reads the input in (see `Stream`, NFR-001). |
| *error tree* | `Genc3.Parser` | (see Specification [AD02], § Terms — not redefined here; modelled by `ErrorTree`) |
| *left-recursive name* | `Genc3.Element` | A *name* of the *definitions table* whose element can reach a *reference* to that same *name* through leftmost positions, every element before the reached *reference* being able to match without consuming input (see `nmsLeftRecursive`). |
| *offset* | `Genc3.Stream` | A number of characters from the start of the input symbol stream, counted from 0. |
| *derivation outcome* | `Genc3.Parser` | The result of one attempt to derive a symbol from the input at one *offset*: either a symbol together with the *offset* after it, or no match. |
| *memo table* | `Genc3.Parser` | The record of *derivation outcomes* already computed, realising *memoisation*. |
| *lookahead* | `Genc3.Symbol` | (see Specification [AD02], § Terms and FR-036 — not redefined here) |
| *examined bound* | `Genc3.Parser` | The exclusive upper bound of the *offsets* a trial read, directly or through reading another *memo table* entry; the source of each *sequence* and *selection* symbol's *lookahead* (see § Function signatures, `Genc3.Parser`). |
| *involved set* | `Genc3.Parser` | The set of in-progress *seeds* a *memo table* entry's trial consulted, directly or through reading another dependent entry (after Warth, Douglass & Millstein 2008); empty for a seed-independent entry (see § Function signatures, `Genc3.Parser`, NFR-001). |
| *extent* | `Genc3.Render` | A measure of a piece of the input symbol stream — its character count, its *line break* count, and the number of characters following its last *line break*. |
| *reused derivation* | `Genc3.Parser` | (see Specification [AD02], § Terms and FR-037 — not redefined here; modelled by `Reuse`) |
| *reuse ground* | `Genc3.Parser` | (see Specification [AD02], § Terms and FR-039 — not redefined here; modelled by `Ground`) |
| *memo switch* | `Genc3.Cli` | (see Specification [AD02], § Terms and IR-012 — not redefined here; modelled by the `Maybe Detail` of `Options`) |
| *detail keyword* | `Genc3.Render` | (see Specification [AD02], § Terms and IR-013 — not redefined here; modelled by `Detail`) |

## Scope

Covers: the module decomposition, data model, and public function interfaces of *product* — the *symbol tree*, the configured syntax, the derivation of symbols including *memoisation* and the automatic handling of left recursion, the positional overwrite of the existing *symbol tree*, the incremental re-parse that seeds the *memo table* from the existing *symbol tree* on both sides of the overwrite — right of it by the suffix argument, left of it by the *lookahead* each serialised symbol records (FR-036) — the *reused derivations* that re-parse reports, the human-readable form, and the CLI effect boundary.

Out of scope: generating an input symbol stream from a *symbol tree*; the implementation bodies of the functions declared here.

## Architecture overview

| Module | Responsibility | Depends on |
| --- | --- | --- |
| `Genc3.Element` | Declares the *syntax element* data model, the *definitions table*, the top-level functions that compose syntax from them, and the up-front check of every *reference*. | - none - |
| `Genc3.Symbol` | Declares the *symbol tree* data model. | `Genc3.Element` |
| `Genc3.Stream` | Declares the *offset*, the width of an overwrite, and the *read position*; reconstructs the input symbol stream from a *symbol tree*, applies the overwrite to it, and derives the *read position* at an *offset*. | `Genc3.Symbol` |
| `Genc3.Syntax` | Configures the required syntax of *product*, as top-level functions, and registers every named *syntax element* in the *definitions table*. | `Genc3.Element` |
| `Genc3.Parser` | Derives symbols from the input symbol stream according to a *syntax element*, resolving each *reference* against the *definitions table*, with *memoisation*, produces the *parser result*, and reports the *reused derivations* the re-parse took over from the existing *symbol tree*. | `Genc3.Element`, `Genc3.Symbol`, `Genc3.Stream` |
| `Genc3.Render` | Declares the `HumanRendering` capability and every instance of it, owns the human-readable form of the *symbol tree* in both directions, of the *error tree*, and of the *reused derivations*, declares the *detail keyword* that selects how much of a *reused derivation* is rendered, and derives the *read position* of every rendered symbol. | `Genc3.Symbol`, `Genc3.Stream`, `Genc3.Parser` |
| `Genc3.Binary` | Owns the *binary form* of the *symbol tree*, in both directions. | `Genc3.Symbol` |
| `Genc3.Cli` | Parses the program arguments into the options *product* runs with. | `Genc3.Stream`, `Genc3.Render` |
| `Main` | The effect boundary: *stdin*, *stdout*, *stderr*, and the *symbol tree* file. | all of the above, `Genc3.Syntax` |

The dependency relation is acyclic: `Element` depends on nothing; `Symbol` and `Syntax` depend only on `Element`; `Stream` and `Binary` on `Symbol`; `Parser` on `Element`, `Symbol`, and `Stream`; `Render` on `Symbol`, `Stream`, and `Parser` (the latter for the `HumanRendering` instances of `ParserResult`, the *error tree*, and the *reused derivations*, which this design places in `Render` with every other instance of the class); `Cli` on `Stream` and `Render`; and `Main` on all of them.

`Cli` depends on `Render` for one type alone: the *detail keyword*, which `eOptions` parses out of the program arguments and which `Render` owns, being the parameter of a rendering rather than of the parse. The alternative — declaring the *detail keyword* in `Cli`, where the argument that selects it arrives — would invert that: `Render` would then depend on `Cli`, and the module owning every human-readable form would take its vocabulary from the module that reads program arguments. The direction chosen keeps the rendering vocabulary in one place and leaves `Cli` what it already is, the translator from arguments into the options *product* runs with. Either direction is acyclic; this one keeps `Render` free of the CLI.

`Genc3.Syntax` is a dedicated module holding nothing but the configured syntax. It is the only module that changes when the required syntax changes, and no module other than `Main` depends on it — `Parser` takes the root *syntax element* and the *definitions table* as arguments rather than importing it. This realises C-002 as a structural property: the grammar lives in source code, in one named place, and is swappable without touching the parser.

## Type declarations

The *symbol tree* is serialised into, and read back from, the same text that FR-018 to FR-023 define as its human-readable form (see § Function signatures, `Genc3.Render`). `Symbol` therefore carries exactly what that form renders — an element kind, an optional *name*, and, for a *terminal char* symbol, the matched character — and nothing besides. A field the rendering has no slot for could not be reconstructed by `mTreeOfRendering`, and the round-trip law stated there would not hold. This is why no symbol carries a *read position*, which is derived and in any case already present in the text; why a *selection* symbol does carry the index of the alternative that matched (FR-029), FR-019 having given the rendering a slot for it; why a *sequence* and a *selection* symbol carry their *lookahead* (FR-036), FR-019 likewise giving it a slot, so that the incremental re-parse can validate reuse left of the overwrite from the serialised tree alone (see `memOfTree`) — a *terminal char* examines exactly its own character and an *empty element* nothing, so their *lookaheads* are constantly zero and neither is carried nor rendered; and why the *name* is carried although it is derivable from the configured syntax — the *symbol tree* is read back without `Genc3.Syntax` in hand, and so must be self-describing.

`Show` is derived for diagnostic output only. It is not the human-readable form of FR-018, which `sRendering` alone produces.

A recursive grammar is expressed through the *reference* (`SynRef`), never by a `Syntax` value embedding itself. A Haskell value that embeds itself (a knot-tied binding) is structurally infinite, and the derived `Ord` of the *memo table*'s keys would diverge comparing it — the memo could then never stop a left-recursive descent (FR-004). `SynRef` removes the need for such values: it truncates the structure at every cite, so each `Syntax` value is a finite — and small — tree, every key comparison terminates, and a left-recursive grammar is written as a *named* element citing itself. `SynRef` is deliberately transparent in the *symbol tree*: deriving a *reference* yields the referenced element's symbol directly, no symbol kind is added, and the human-readable form of FR-018 to FR-029 — and with it the round-trip law of `mTreeOfRendering` — is untouched.

The *definitions table* is the mapping a *reference* is resolved against. It is configured by `Genc3.Syntax` alongside the root *syntax element* (see § Function signatures) and passed to the parser as an argument, keeping the C-002 property intact.

`Genc3.Render` declares the type class below together with every one of its instances. '02_04_coding.md' §3 groups a data type with its instances, which assumes the instance's class is declared in the same module; here the class is local and the instantiated types are not, so the instances group with the class instead. None of them is an orphan instance, the class being local to the module that declares them.

```haskell
-- HumanRendering ...
{-| The capability of having a form meant to be read by a human.

* the rendered form is the sole human-readable form of the value; 'show' is diagnostic output and is not this
* for a 'SymbolTree' the rendered form is also the serialised form (see 'mTreeOfRendering')
-}
class HumanRendering a where
    -- realises FR-018, IR-005, IR-006
    sRendering
        :: a
        -> String -- ^ the human-readable form of the value


-- realises FR-017, FR-021, FR-023
-- prefix: nm
type Name = String


-- realises FR-003, FR-004, FR-006
-- prefix: syn
data Syntax
    = SynChar Char                 -- ^ a *terminal char*
    | SynSeq (Maybe Name) [Syntax] -- ^ a *sequence*, optionally named
    | SynSel (Maybe Name) [Syntax] -- ^ a *selection*, optionally named
    | SynEmpty (Maybe Name)        -- ^ an *empty element*, optionally named
    | SynRef Name                  -- ^ a *reference* to the named *syntax
                                   --   element* of the *definitions table*
    deriving (Eq, Ord, Show)


-- realises FR-004
-- prefix: defs
newtype Definitions = Definitions (Map Name Syntax)
    deriving (Eq, Show)


-- realises FR-006, FR-008, FR-009, FR-010, FR-011, FR-017, FR-029, FR-036
-- prefix: sym
data Symbol
    = SymChar Char                   -- ^ from a *terminal char*; no child symbols
    | SymSeq (Maybe Name) [Symbol]
        Int                          -- ^ from a *sequence*; one child per matched
                                     --   element, and the *lookahead*
    | SymSel (Maybe Name) Int Symbol
        Int                          -- ^ from a *selection*; the index of the matched
                                     --   alternative, counted from 0, exactly one
                                     --   child, and the *lookahead*
    | SymEmpty (Maybe Name)          -- ^ from an *empty element*; no child symbols
    deriving (Eq, Show)


-- realises FR-002
-- prefix: symt
data SymbolTree
    = SymtEmpty       -- ^ the empty *symbol tree*
    | SymtRoot Symbol -- ^ a *symbol tree* with a root symbol
    deriving (Eq, Show)

-- realises FR-018, FR-022, IR-004, IR-005
instance HumanRendering SymbolTree


-- realises IR-002
-- prefix: off
newtype Offset = Offset Int
    deriving (Eq, Ord, Show)


-- realises NFR-001
-- prefix: strm
newtype Stream = Stream (Array Int Char)
    deriving (Eq, Show)


-- realises IR-002
-- prefix: wdt
newtype Width = Width Int
    deriving (Eq, Ord, Show)


-- realises FR-016
-- prefix: rp
data ReadPos = ReadPos
    Int -- ^ line number, counted from 1
    Int -- ^ column number, counted from 1
    deriving (Eq, Show)

-- realises FR-016, FR-019
instance HumanRendering ReadPos


-- realises FR-024, FR-025, FR-026
-- prefix: ext
data Extent = Extent
    Int -- ^ number of characters
    Int -- ^ number of *line breaks*
    Int -- ^ number of characters following the last *line break*
    deriving (Eq, Show)

instance Semigroup Extent
instance Monoid Extent


-- realises FR-005, FR-007, FR-012, FR-014
-- prefix: out
data Outcome
    = OutMatched Symbol Offset -- ^ the derived symbol, and the *offset* after it
    | OutNoMatch ErrorTree     -- ^ the *syntax element* does not match at this
                               --   *offset*: what did not match, and where
    deriving (Eq, Show)


-- realises FR-014, FR-031, FR-032, FR-033, FR-034
-- prefix: err
data ErrorTree = ErrorTree
    ErrorKind         -- ^ what did not match
    ReadPos           -- ^ the *read position* at which its matching began
    (Maybe ErrorTree) -- ^ the child error, where the failure has one — a
                      --   *sequence*'s first non-matching element (FR-032)
    deriving (Eq, Show)

-- realises FR-031, FR-034, FR-035
instance HumanRendering ErrorTree


-- realises FR-031, FR-034
-- prefix: erk
data ErrorKind
    = ErkChar Char        -- ^ a *terminal char* did not match
    | ErkSeq (Maybe Name) -- ^ a *sequence* did not match
    | ErkSel (Maybe Name) -- ^ no alternative of a *selection* matched
    | ErkRef Name         -- ^ a *reference* did not match
    | ErkUnconsumed       -- ^ the root matched, input remains (FR-034)
    deriving (Eq, Show)


-- realises FR-004, FR-005, FR-036, NFR-001, NFR-003, NFR-004
-- prefix: mem
data Memo = Memo
    (IntMap (Map Syntax (Outcome, Set (Offset, Name), Int)))
                         -- ^ the recorded *derivation outcomes*: the
                         --   *offset*'s number, then the *syntax
                         --   element*, each entry with its *involved set*
                         --   and its *examined bound*
    (Set (Offset, Name)) -- ^ the *references* whose growth is in
                         --   progress
    (Set (Offset, Name)) -- ^ the in-progress *seeds* the trial currently
                         --   running has consulted
    Int                  -- ^ the read watermark of the trial currently
                         --   running: the exclusive upper bound of the
                         --   *offsets* it has read
    deriving (Eq, Show)


-- realises FR-037, FR-038
-- prefix: reu
data Reuse = Reuse
    Offset -- ^ the *offset* at which it applies in the overwritten input
           --   symbol stream
    Syntax -- ^ the *syntax element* its symbol was derived from — the
           --   *memo table* key it is seeded under
    Symbol -- ^ the symbol taken over from the existing *symbol tree*
    Offset -- ^ the *offset* after that symbol's matched input
    Ground -- ^ its *reuse ground*
    deriving (Eq, Show)


-- realises FR-039
-- prefix: grd
data Ground
    = GrdShifted Int -- ^ it applies at or beyond the end of the overwritten
                     --   region: the number of *offsets* by which the
                     --   overwrite shifts it
    | GrdBounded Int -- ^ it applies before the insertion position: the
                     --   *offset* after its symbol's matched input plus
                     --   that symbol's *lookahead*, at most the insertion
                     --   position
    deriving (Eq, Show)


-- realises FR-027, FR-028
-- prefix: gd
type Guides = [Bool]


-- realises IR-013
-- prefix: det
data Detail
    = DetBasic     -- ^ `basic`: the *offset*, the element, and the *offset*
                   --   after the matched input (FR-042)
    | DetLookahead -- ^ `lookahead`: and the symbol's *lookahead* (FR-043)
    | DetGround    -- ^ `ground`: and the *reuse ground* (FR-044, FR-045)
    deriving (Eq, Ord, Show)


-- realises FR-040, FR-041
-- prefix: reus
data Reuses = Reuses
    Detail  -- ^ how much of each line is rendered
    [Reuse] -- ^ the *reused derivations*, in the order they are rendered
    deriving (Eq, Show)

-- realises FR-040, FR-041, FR-042, FR-043, FR-044, FR-045, IR-015
instance HumanRendering Reuses


-- realises FR-013, FR-014
-- prefix: prs
data ParserResult
    = PrsError ErrorTree -- ^ the input symbol stream does not match the
                         --   configured syntax: the *error tree* (FR-014)
    | PrsTree SymbolTree -- ^ the resulting *symbol tree*
    deriving (Eq, Show)

-- realises FR-013, FR-014, IR-005, IR-006
instance HumanRendering ParserResult


-- realises IR-002, IR-003, IR-008, IR-012, IR-013, IR-014
-- prefix: opt
data Options = Options
    Offset   -- ^ the insertion position
    Width    -- ^ the number of symbols to overwrite
    FilePath -- ^ the path of the serialised *symbol tree*
    Bool     -- ^ whether the *symbol tree* file is human-readable (IR-008)
    (Maybe Detail)
             -- ^ the *detail keyword* where the *memo switch* is present,
             --   'DetBasic' where it carries none (IR-014); nothing where
             --   the switch is absent (IR-012, IR-016)
    deriving (Eq, Show)
```

## Function signatures

All functions in this section are pure; see § Purity and effect boundaries.

### `Genc3.Element`

```haskell
-- realises FR-003
synTerminalChar :: Char -> Syntax

-- realises FR-003
synSequence :: [Syntax] -> Syntax

-- realises FR-003, FR-017
synNamedSequence :: Name -> [Syntax] -> Syntax

-- realises FR-003
synSelection :: [Syntax] -> Syntax

-- realises FR-003, FR-017
synNamedSelection :: Name -> [Syntax] -> Syntax

-- realises FR-003
synEmpty :: Syntax

-- realises FR-003, FR-017
synNamedEmpty :: Name -> Syntax

-- realises FR-004
synReference :: Name -> Syntax

-- realises FR-004
eDefsChecked
    :: Definitions               -- ^ the *definitions table* to check
    -> Either String Definitions -- ^ the table, or a message for *stderr*
                                 --   naming a dangling *reference*

-- realises FR-004, NFR-001
nmsLeftRecursive
    :: Definitions -- ^ the *definitions table* of the configured syntax
    -> Set Name    -- ^ its *left-recursive names*
```

`synReference` composes a *reference*; `eDefsChecked` verifies, once and up front, that every *reference* reachable from the *definitions table*'s elements names an entry of that table — a dangling *reference* is a configuration error, reported before any parsing rather than surfacing as a misleading no-match in its middle.

`nmsLeftRecursive` computes, once per *definitions table*, which *names* are *left-recursive names*: a nullability fixed point over the table (an element can match without consuming input), then a leftmost-reachability closure over it. The parser grows the *seed* only at *references* citing these names (see § `Genc3.Parser`). The analysis errs safely in both directions: over-approximating costs only performance, and under-approximating costs only the match of a left-recursive alternative — never termination, which rests on the seeding every trial gets regardless.

### `Genc3.Syntax`

```haskell
-- realises C-002
synRoot :: Syntax

-- realises C-002, FR-004
defsSyntax :: Definitions
```

Every further top-level function of this module composes one named *syntax element*. The function itself is named with the `syn` prefix, per '02_04_coding.md' §2; the *name* carried by the element it composes is that function's name with the prefix removed and the following letter lowercased:

```haskell
-- realises FR-017
synDigit :: Syntax
synDigit = synNamedSelection "digit"
    [ synTerminalChar '1', synTerminalChar '2', synTerminalChar '3' ]
```

`defsSyntax` registers every named *syntax element* of this module — `synRoot`'s included — under its *name*, and is the *definitions table* the parser resolves each *reference* against. A recursive element cites itself (or a mutually recursive partner) through `synReference` rather than by embedding the binding, keeping every `Syntax` value finite. Left recursion is written directly — the parser grows its *seed* to the full match (see § `Genc3.Parser`), deriving a left-associative *symbol tree*:

```haskell
-- realises FR-017
synDigits :: Syntax
synDigits = synNamedSelection "digits"
    [ synSequence [ synReference "digits", synDigit ], synDigit ]
```

A rendered symbol line therefore reads `Sel(digit):1:1` (FR-019, FR-021) — the grammar's own vocabulary, rather than the source code's naming convention. FR-017's "named after the top-level function that composes it" holds through that one mechanical rule, which maps each way without ambiguity: `Sel(digit)` locates `synDigit` in this module, and no other function of this module can claim that name.

The rule is a convention of this module, not a property the compiler checks — Haskell gives a binding no access to its own name. The Implementation Phase can cover it by asserting the rendered identifier of each named *syntax element* against its expected string.

### `Genc3.Stream`

```haskell
sOfTree :: SymbolTree -> String

-- realises FR-001, IR-002
sOverwritten
    :: Offset -- ^ the insertion position
    -> Width  -- ^ the number of symbols to overwrite
    -> String -- ^ the input symbol stream
    -> String -- ^ the existing input symbol stream
    -> String -- ^ the resulting input symbol stream

-- realises FR-015, FR-016, FR-031
rpAt
    :: Stream  -- ^ the *indexed stream*
    -> Offset  -- ^ the *offset* to locate
    -> ReadPos -- ^ the *read position* of that *offset*

-- realises NFR-001
strmOfString
    :: String -- ^ the input symbol stream
    -> Stream -- ^ the *indexed stream*, converted in one pass

-- realises NFR-001
mChAt
    :: Stream     -- ^ the *indexed stream*
    -> Offset     -- ^ the *offset* to read at
    -> Maybe Char -- ^ the character there, or nothing past the end

-- realises NFR-001
nLengthOfStream
    :: Stream -- ^ the *indexed stream*
    -> Int    -- ^ its number of characters, in constant time
```

`sOfTree` realises no requirement of its own; it is the internal helper '02_02_design.md' §5 permits to carry no requirement ID. It relies on the concatenation of a *symbol tree*'s *terminal char* symbols being the input symbol stream that tree was derived from — which holds because only a *terminal char* consumes input (FR-008), a *sequence*, *selection* and *empty element* symbol contributing none of their own (FR-009 to FR-011).

### `Genc3.Parser`

```haskell
-- realises FR-001, FR-013, FR-037, IR-015
prsResult
    :: Definitions -- ^ the *definitions table* of the configured syntax
    -> Syntax      -- ^ the root *syntax element* of the configured syntax
    -> SymbolTree  -- ^ the existing *symbol tree*
    -> Offset      -- ^ the insertion position
    -> Width       -- ^ the number of symbols to overwrite
    -> String      -- ^ the input symbol stream
    -> (ParserResult, [Reuse])
                   -- ^ the *parser result*, and the *reused derivations*
                   --   the re-parse took over

-- realises FR-005
mOutAt
    :: Memo          -- ^ the *memo table*
    -> Offset        -- ^ the *offset* of the trial
    -> Syntax        -- ^ the *syntax element* of the trial
    -> Maybe Outcome -- ^ the recorded outcome, where there is one

-- realises FR-005
memInserted
    :: Offset  -- ^ the *offset* of the trial
    -> Syntax  -- ^ the *syntax element* of the trial
    -> Outcome -- ^ the outcome to record
    -> Memo    -- ^ the *memo table* before
    -> Memo    -- ^ the *memo table* with the outcome recorded

-- realises FR-004, FR-005, FR-007, FR-008, FR-009, FR-010, FR-011, FR-012
outDerivation
    :: Definitions -- ^ the *definitions table* of the configured syntax
    -> Set Name    -- ^ its *left-recursive names* (see `nmsLeftRecursive`)
    -> Stream      -- ^ the whole input symbol stream, as *indexed stream*
    -> Syntax      -- ^ the *syntax element* to attempt
    -> Offset      -- ^ the *offset* to attempt it at
    -> Memo        -- ^ the *memo table* before the trial
    -> (Outcome, Memo)
```

`prsResult` computes the *left-recursive names* once, via `nmsLeftRecursive`, and threads them through every trial.

```haskell
-- realises FR-001, FR-037, FR-038, FR-039, NFR-002, NFR-003, NFR-004
memOfTree
    :: Definitions -- ^ the *definitions table* of the configured syntax
    -> Offset      -- ^ the insertion position of the overwrite
    -> Width       -- ^ the number of symbols the overwrite replaces
    -> Int         -- ^ the number of symbols the overwrite inserts
    -> SymbolTree  -- ^ the existing *symbol tree*
    -> (Memo, [Reuse])
                   -- ^ the seeded *memo table*, and one *reused
                   --   derivation* per symbol it seeded, in rendering
                   --   order
```

`memOfTree` yields the *reused derivations* from the very traversal that seeds, rather than from a second pass: the two must agree, or the displayed reuse would describe something other than what the parse actually reused, and one traversal cannot disagree with itself. This is why the pair replaces the bare `Memo`, and why `prsResult` passes the list on rather than recomputing it — `Main` never calls `memOfTree` itself, so the *reused derivations* it writes are the ones the *parser result* beside them was derived against.

One *reused derivation* is reported per seeded symbol, not per *memo table* entry. A named symbol seeds two keys — the *reference* `SynRef nm` and the body element registered under that *name* — because either may be the key a trial arrives at; both stand for the one derivation of the one symbol, and reporting them twice would misrepresent the reuse as double what it is. The `Syntax` a `Reuse` carries (FR-038) is that symbol's own element: the `SynChar` of a *terminal char*, and the body element of a named one.

The traversal already emits the order FR-040 requires, so nothing is sorted. It visits the existing *symbol tree* depth-first, which is ascending in the *offsets* of the tree it visits; the seeded *offsets* stay ascending under the overwrite because every left-of-overwrite symbol satisfies its *offset* plus width plus *lookahead* at most the insertion position, and so begins before it, while every right-of-overwrite symbol begins at or after the end of the overwritten region and is shifted to at or after the insertion position plus the inserted width. Symbols sharing an *offset* — an *empty element* symbol has zero width, so its following sibling begins where it does — are emitted in the depth-first order that FR-040 names as the tie-break.

Where the *memo switch* is absent, no `Reuse` is built: the list is consumed only by the rendering `Main` skips (IR-016), and Haskell's laziness leaves the cells unevaluated. Making the reuse visible therefore costs nothing on a run that does not ask for it, which is what keeps the pair compatible with NFR-002 to NFR-004.

`memOfTree` realises the incremental re-parse: `prsResult` derives against the *memo table* it seeds from the existing *symbol tree* instead of the empty one. The seeding rests on one property: a *derivation outcome* at an *offset* is a pure function of the *definitions table* and the input suffix from that *offset*. An overwrite of the region `[p, p+w)` leaves every suffix at old *offsets* at or beyond `p+w` unchanged up to a shift by the length difference — so every symbol of the existing tree starting there is, at its shifted *offset*, a valid *derivation outcome* of the new stream, errors and all: whatever the fresh derivation would compute for a seeded key, it would compute equal to the seed, so the incremental result is identical to the from-scratch result by construction.

Left of the overwrite, the suffix argument is unavailable — a suffix there contains the overwrite — but a sharper one holds: a derivation only depends on the input it actually read. A symbol whose *offset* plus width plus *lookahead* is at most the insertion position was derived reading only characters the overwrite leaves in place, so its outcome transfers at its unshifted *offset*: the fresh derivation of its key would read the same characters and compute the same outcome, step by step (NFR-003, NFR-004). One exclusion keeps this sound in the presence of the growing of *seeds*: a symbol embedded by a superseded growth iteration is not the fixed point its memo key derives, and its recorded *lookahead* reflects only that iteration's reads — such a symbol always starts at its growth's *offset*, as does every symbol on the path from the grown *reference*'s symbol down to it, its parent included. A named symbol is therefore left-seeded only where it does not start at the same *offset* as its parent symbol; a *terminal char*, which examines exactly its own character, is exempt. The widths the conditions need are annotated onto the tree once, bottom-up, before the traversal — recomputing them per visited symbol would cost the square of a deep spine.

Two practicalities bound the mechanism. Only the topmost reusable symbols are seeded — a memo hit on a parent spares every descendant. And only a *terminal char* or a *named* element can reconstruct its memo key (the *terminal char*'s element is its character; a named element's is its *name*'s entry, seeded under both the *reference* key and the body key); an unnamed interior element's *syntax element* is not recoverable from its symbol, so its children are visited instead. The gain follows the tree's shape: an overwrite at the beginning of a stream of many self-contained units — parenthesised sub-formulas — re-parses about an order of magnitude faster than a full parse, while a single left-recursive spine, whose every interesting symbol starts at *offset* 0, re-grows its spine whichever side the overwrite falls on — reuse left of the overwrite spares it the re-derivation of its per-character leaves, not the spine itself. NFR-002 pins the begin-of-stream case at 1 second; NFR-003 and NFR-004 pin the middle and end cases at 1 second each — half the full-parse budget of NFR-001, which reuse on both sides of the overwrite sustains even on the spine (measured ≈ 0.45 seconds against a 0.70-second full parse). The seeding assumes the existing *symbol tree* stems from the current configured syntax and the current serialised forms; after the configured syntax — or, as with the introduction of the *lookahead*, the serialised form — changes, existing *symbol tree* files must be discarded.

`prsResult` keeps the plain `String` at its boundary and converts it once, via `strmOfString`, before deriving; every trial below it reads the *indexed stream*. This is what realises NFR-001: a *terminal char* trial reads its character in constant time through `mChAt`, where walking a plain string to an *offset* would cost O(*offset*) per trial and make the whole parse quadratic in the input length.

The *memo table* is keyed on an *offset* and a *syntax element*, as FR-005 requires, in two levels: an `IntMap` on the *offset*'s number — the discriminating component, and the cheap comparison — holding one small per-*offset* map on the element. `Syntax` derives `Ord`, so a *syntax element* is its own key and needs no identity of its own: structural equality is the correct notion here, two structurally identical elements deriving the same outcome at the same *offset*. Comparing element keys costs O(size of the *syntax element*) rather than O(1), a cost the design accepts because `SynRef` keeps every element small and the per-*offset* maps stay tiny; giving each element an integer identity would reduce it further, at the price of a numbering pass — the known upgrade path should profiling ever demand it. `mOutAt` and `memInserted` are the two access paths, so no caller touches the representation.

Before a trial runs, `outDerivation` records its key with `OutNoMatch` — the *seed* — and replaces it with the trial's actual `Outcome` afterwards. This is what makes FR-004 hold: a left-recursive descent reaches the same *syntax element* at the same *offset* a second time — necessarily through a *reference*, a finite `Syntax` value being unable to embed itself — finds the *seed*, and stops instead of recursing. A *reference* is memoised on its own key before it is resolved, so a repeat attempt at an *offset* costs one small key comparison and no *definitions table* lookup.

Termination alone would leave a left-recursive alternative itself unmatched: the innermost re-entry yields the seeded no-match, so a single derivation of `digits = digits digit | digit` consumes exactly one digit. `outDerivation` therefore *grows* the *seed* at each *reference* citing a *left-recursive name* (after Warth, Douglass & Millstein, *Packrat Parsers Can Support Left Recursion*, PEPM 2008, simplified for this design): where such a *reference*'s body derives a match, that outcome becomes the new *seed* and the body is re-derived against it, repeatedly, until an iteration no longer extends the match; that fixed point is the *reference*'s *derivation outcome*. Each growth iteration extends the matched *offset* strictly, so a *reference* grows at most once per remaining input character, and nested growings are bounded the same way — left recursion, direct and indirect, is handled automatically, with no annotation in the configured syntax.

A *reference* citing a name that is not *left-recursive* takes the plain seed-and-record path of every other element, its body's sub-derivations persisting in the *memo table*. This confinement is what NFR-001 demands: growing every *reference* would re-derive each *reference*'s body once more to confirm its fixed point, and under the wholesale discarding of a superseded iteration's *memo table* additions that preceded the *involved set* (below), that re-derivation doubled the work at every nesting level of the configured syntax — measured as a 32-fold re-derivation of the digit chain on the configured PEMDAS syntax, breaching NFR-001 seven-fold, where confining the growth left a four-fold factor and met it.

Two properties keep the growing sound in the pure setting. First, no outcome computed against a superseded *seed* survives into the *memo table* handed onward — each entry carries its *involved set*, and a superseded iteration's dependent entries are purged with it (next paragraph). Second, every cycle of the configured syntax passes through a *reference* citing a *left-recursive name* — each member of a leftmost cycle reaches itself through it — so the growth is met exactly where re-entry can happen, while the four structural elements and the non-left-recursive *references* keep the plain seed-and-record mechanism unchanged.

Alongside the *involved set*, every entry records its *examined bound*, the source of the *lookahead* (FR-036): the *memo table* carries the read watermark of the trial currently running, raised by every *terminal char* read — matched or not, at or past the end of the input — and, on a memo hit, by the read entry's own *examined bound*, a reuse standing for the reads that produced the entry. Each trial runs with the watermark reset and joins it back afterwards, so its recorded entry's *examined bound* covers exactly its own reads; a *sequence* or *selection* symbol is stamped with the bound minus its end — its *lookahead* — as it is recorded. At a growing *reference*, the *seed* carries the maximum watermark of every iteration so far, and the final entry's bound is the maximum over all of them. A *terminal char* or *empty element* trial makes no recursive call — no re-entry can reach it — so it plants no *seed* and is recorded directly, sparing the parse's hottest path a second insertion (NFR-001).

The growth's cost is bounded by per-entry dependency tracking — Warth's involved sets (NFR-001). The *memo table* records which *references*' growth is in progress, and every trial runs with the consultation accumulator reset, so the entry it records carries as its *involved set* exactly the in-progress *seeds* that trial consulted: directly, or through reading another entry whose *involved set* names them — reading a dependent entry is depending on the *seeds* it depends on. An entry whose *involved set* is empty is seed-independent, and its recording is unconditional. Three consequences carry NFR-001. Where a growth's first iteration never consulted its own *seed*, its outcome is already the fixed point and no further iteration runs. Where an iteration is superseded, its additions are purged by *involved set* rather than discarded wholesale: only the entries depending on the superseded *seed* are dropped, every seed-independent sub-result surviving into the next iteration — and since a trial only ever reads keys at or beyond its own *offset*, and a *seed* only at its own, every dependent entry sits at the growing *reference*'s *offset*, making the purge a filter of one small per-*offset* map. And the iteration confirming the fixed point derived against the very *seed* the fixed point confirms, so its additions — the dependent ones included — are computed against the final value and carry onward. Measured effect: the input of 100000 characters of parenthesised sub-formulas, whose genuinely re-entered growth at the formula's root previously re-derived the whole stream once more (about 3.4 seconds, breaching NFR-001), parses in about 1.6 seconds. The *symbol tree* a left-recursive rule derives is left-associative — `digits = digits digit | digit` applied to `123` nests `(1 2)` under `((1 2) 3)` — which is the shape an expression grammar wants and a right-recursive rewrite could not deliver.

The parser constructs each error's *read position* through `rpAt`, whose scan of the input up to the *offset* costs O(*offset*). Failed trials are frequent — every rejected alternative of a *selection* is one — so the `ReadPos` inside an `ErrorTree` is deliberately left to Haskell's laziness: the thunk is forced only for the one error path a *parsing error* finally surfaces, never for the failures that memoisation and backtracking absorb, keeping NFR-001 intact.

### `Genc3.Render`

```haskell
-- realises FR-018, FR-027
sSymbolRendering
    :: Guides           -- ^ one flag per ancestor beyond the root, outermost first;
                        --   True where that ancestor has a following sibling symbol
    -> Extent           -- ^ the *extent* of the input preceding this symbol
    -> Symbol
    -> (String, Extent) -- ^ the rendered lines, and the *extent* after this symbol

-- realises FR-019, FR-036
sLineOfSymbol :: Extent -> Symbol -> String

-- realises FR-027, FR-028
sPrefixOfGuides :: Guides -> String

-- realises FR-020
sElemKind :: Symbol -> String

-- realises FR-021, FR-023
sElemId :: Symbol -> String

-- realises FR-019, FR-029
sIndexOfSymbol :: Symbol -> String

-- realises FR-015, FR-016, FR-024, FR-025, FR-026
rpOfExtent :: Extent -> ReadPos

-- realises FR-024, FR-025, FR-026
extOfChar :: Char -> Extent

-- realises IR-003
mTreeOfRendering :: String -> Maybe SymbolTree

-- realises IR-003, IR-007
mTreeOfExisting
    :: Maybe String     -- ^ the *symbol tree* file's content, or nothing
                        --   where that file does not exist
    -> Maybe SymbolTree -- ^ the existing *symbol tree*, or nothing where
                        --   the content is not a well-formed rendering

-- realises FR-042, FR-043, FR-044, FR-045
sLineOfReuse
    :: Detail -- ^ how much of the line to render
    -> Reuse  -- ^ the *reused derivation* to render
    -> String -- ^ its rendered line, without a trailing *line break*

-- realises FR-044, FR-045
sGround :: Ground -> String

-- realises IR-013
mDetailOfKeyword
    :: String       -- ^ the argument following the *memo switch*
    -> Maybe Detail -- ^ the *detail keyword* it names, or nothing where it
                    --   names none
```

`mTreeOfExisting` maps the absence of the *symbol tree* file to the empty *symbol tree* (IR-007) and its content to `mTreeOfRendering` otherwise. It exists so that `Main` decides nothing of its own: whether a missing file is an error or an empty tree is a property of *product*, testable without I/O, rather than a branch buried in the effect boundary. Its result is `Nothing` only for a file that exists and is malformed — the case IR-004 leaves unchanged.

`mDetailOfKeyword` is the sole place the three keyword spellings of IR-013 are known. It sits here rather than in `Genc3.Cli` because the keyword names a degree of rendering, and the module that owns the degrees owns the vocabulary that names them; `eOptions` applies it and reports an unknown keyword as `Left`, distinguishing the argument that names no degree from the absent argument that means `DetBasic` (IR-014). `Detail` derives `Ord` in the degree's own order, so `sLineOfReuse` decides what to append by comparison rather than by enumerating cases — the `lookahead` field belongs to every degree from `DetLookahead` upward (FR-043).

The `Reuses` instance renders the heading line of FR-041 and then one `sLineOfReuse` per *reused derivation*, in the order the list already carries (FR-040, see `memOfTree`); it renders the heading even where the list is empty, so a run that reused nothing says so rather than falling silent. Unlike the `SymbolTree` instance, this rendering has no counterpart reading it back: the *reused derivations* are diagnostic output, not a serialised form, so no round-trip law is claimed for them and none of the constraints that shape `Symbol` applies to `Reuse` — it carries the whole `Symbol` and its `Syntax` key precisely because nothing has to reconstruct it from its text.

`sLineOfSymbol` renders a line's body without any prefix, so the `SymbolTree` instance emits the root symbol's line from it directly and realises FR-022 by prefixing nothing. Every other line is produced by `sSymbolRendering`, whose `Guides` argument grows by one flag per level: `True` while the ancestor still has a following sibling, `False` once it does not. `sPrefixOfGuides` turns that into `| ` and two-space runs, closed by `+-`.

Every guide is derivable from the *symbol tree*'s own shape, so the prefix adds no information the round-trip has to recover: reading back, the count of two-character groups before `+-` gives the depth, and the tree structure follows from that alone.

`sSymbolRendering` returns the *extent* after the symbol it rendered, so the `SymbolTree` instance threads one accumulator through a single depth-first traversal and derives every *read position* in one pass over the *symbol tree*. No function recomputes the *extent* of a subtree it has already rendered.

`sRendering` at `SymbolTree` and `mTreeOfRendering` are inverse: `mTreeOfRendering (sRendering t) == Just t` for every `t :: SymbolTree`, the empty *symbol tree* included. The serialised form required by IR-003 and IR-004 is therefore the same form the human reads under IR-005 — *product* has one textual form, not two.

### `Genc3.Binary`

```haskell
-- realises FR-030, IR-009
bsOfTree :: SymbolTree -> ByteString

-- realises FR-030, IR-011
mTreeOfBinary :: ByteString -> Maybe SymbolTree

-- realises IR-007, IR-011
mTreeOfExistingBinary
    :: Maybe ByteString -- ^ the *symbol tree* file's bytes, or nothing
                        --   where that file does not exist
    -> Maybe SymbolTree -- ^ the existing *symbol tree*, or nothing where
                        --   the bytes are not a well-formed *binary form*
```

`bsOfTree` and `mTreeOfBinary` are inverse: `mTreeOfBinary (bsOfTree t) == Just t` for every `t :: SymbolTree` (FR-030), the empty *symbol tree* included — the same law `sRendering` and `mTreeOfRendering` satisfy for the human-readable form. Bytes that are not a well-formed *binary form* — a truncated encoding, an unknown tag, trailing content — yield `Nothing`, reaching `Main` exactly as a malformed human-readable file does. `mTreeOfExistingBinary` mirrors `mTreeOfRendering`'s companion `mTreeOfExisting`: the absence of the file is the empty *symbol tree* (IR-007), whichever form is selected.

The encoding itself is one tag byte per constructor followed by its fields, lengths and indices as fixed-width non-negative integers and each character by its code point — compact, and total in both directions. Unlike the human-readable form, it costs O(size of the *symbol tree*) to write: no per-line guide prefix, whose accumulation makes the human-readable rendering O(depth²) on the deep trees the configured syntax derives.

### `Genc3.Cli`

```haskell
-- realises IR-002, IR-003, IR-008, IR-012, IR-013, IR-014
eOptions :: [String] -> Either String Options
```

`eOptions` accepts the *memo switch* in both spellings, `--show-memo` and `-sm` (IR-012), and the *detail keyword* as the argument that may follow it (IR-013). The keyword is optional, so the switch is the one argument of *product* whose successor may or may not belong to it: an argument following it that `mDetailOfKeyword` recognises is consumed as the keyword, and anything else — another switch, or the end of the arguments — leaves the degree at `DetBasic` (IR-014) and is parsed on its own terms. An argument that begins with a dash is never taken as the keyword, so a mistyped keyword cannot swallow the switch that follows it; it is reported as `Left` instead.

### `Main`

```haskell
-- realises IR-001, IR-004, IR-005, IR-006, IR-007, IR-009, IR-010, IR-011,
--          IR-015, IR-016, IR-017
main :: IO ()
```

`main` writes the *reused derivations* to *stdout* after the *symbol tree* of IR-005, and only where the *parser result* is a *symbol tree* and the *memo switch* is present (IR-015). A *parsing error* writes none (IR-017): the error goes to *stderr* under IR-006, and *stdout* stays empty, so nothing follows a *symbol tree* that was never written. An absent switch writes none either (IR-016). The decision is a pattern match on the `Maybe Detail` of `Options` against the `ParserResult`, both already in hand — `main` renders through `sRendering` at `Reuses` and adds no logic of its own, as it adds none for any other output.

## Data flow

| # | Stage | Type in | Type out | Module |
| --- | --- | --- | --- | --- |
| 1 | Read the program arguments | `[String]` | `Either String Options` | `Genc3.Cli` |
| 2 | Check every *reference* of the configured syntax | `Definitions` | `Either String Definitions` | `Genc3.Element` |
| 3 | Read the input symbol stream from *stdin* | — | `String` | `Main` |
| 4 | Read the *symbol tree* file, where it exists, in the form the *human-readable switch* selects | `FilePath` | `Maybe String` or `Maybe ByteString` | `Main` |
| 5 | Read the existing *symbol tree* back, the empty one where there is no file | `Maybe String` or `Maybe ByteString` | `Maybe SymbolTree` | `Genc3.Render` or `Genc3.Binary` |
| 6 | Reconstruct the existing input symbol stream | `SymbolTree` | `String` | `Genc3.Stream` |
| 7 | Overwrite the given width at the given position | `String` | `String` | `Genc3.Stream` |
| 8 | Seed the *memo table* with the existing *symbol tree*'s reusable derivations, reporting what was seeded | `SymbolTree` | `(Memo, [Reuse])` | `Genc3.Parser` |
| 9 | Derive the resulting *symbol tree*, against the seeded *memo table* | `String` | `(ParserResult, [Reuse])` | `Genc3.Parser` |
| 10 | Render the *parser result* via `sRendering` | `ParserResult` | `String` | `Genc3.Render` |
| 11 | Render the *reused derivations* via `sRendering`, where the *memo switch* is present and the *parser result* is a *symbol tree* | `Reuses` | `String` | `Genc3.Render` |
| 12 | Write to *stdout* or *stderr*, and write the file on success, in the form the *human-readable switch* selects | `String` | — | `Main` |

*Stdout* always carries the human-readable form (IR-005); the *human-readable switch* selects only the *symbol tree* file's form (IR-009 to IR-011), `Main` picking `Render`'s or `Binary`'s pure pair accordingly.

Stages 6 to 9 are the incremental re-parse: the existing *symbol tree* is turned back into the input it was derived from, the overwrite is applied to that input, and the result is re-derived — against a *memo table* seeded with the tree's reusable derivations (stage 8), so the derivation revisits only what the overwrite could have changed. The *symbol tree* is the state that persists between runs; the input symbol stream and the reusable derivations are both reconstructed from it on each run.

Stage 11 is what the *memo switch* makes visible: the very seeding of stage 8, reported as data and rendered for a human. It is the only stage the switch adds, and the only one that may be skipped — stages 1 to 10 run identically whether the switch is given or not, so what the switch shows is what the parse did, not a second parse performed to show it.

## Error handling strategy

- Expected failure is represented in types. A failed derivation is `OutNoMatch` carrying its *error tree* and reaches the caller as `PrsError` (FR-013, FR-014): a *terminal char* mismatch is a leaf, a failed *sequence* chains the error of its first non-matching element (FR-032), a failed *selection* ends the chain (FR-033), and input left over after the root's match is the single `ErkUnconsumed` error (FR-034). The *seed* of a trial in progress carries the trial's own element as a childless error, which surfaces only where a left-recursive *reference* genuinely never matches.
- A *symbol tree* file whose content is not a well-formed rendering yields `Nothing` from `mTreeOfRendering`; `Main` reports it and leaves the file unchanged, consistent with IR-004. A file whose bytes are not a well-formed *binary form* yields `Nothing` from `mTreeOfBinary` and is treated identically.
- A *symbol tree* file that does not exist is not a failure: `Main` reads it as no content, and `mTreeOfExisting` yields the empty *symbol tree* (IR-007). Every other I/O failure of reading that file — it exists but cannot be read — is reported on *stderr*, so the two cases stay distinguishable rather than being collapsed into one catch-all.
- Malformed program arguments yield `Left` from `eOptions`, carrying a message for *stderr*. An argument following the *memo switch* that begins with a dash is not a *detail keyword* but the next switch, leaving the degree at `DetBasic` (IR-014); one that does not begin with a dash and names no degree is a mistyped keyword and yields `Left`, rather than being silently ignored or consumed as another argument's value.
- A dangling *reference* — a `SynRef` naming no entry of the *definitions table* — yields `Left` from `eDefsChecked`, carrying a message for *stderr*; the check runs once, before any parsing (§ Data flow, stage 2), so a dangling *reference* cannot occur during derivation.
- No partial function is used: `head`, `fromJust`, `read`, and incomplete pattern matches are excluded. `SymSel` holds exactly one child by construction, and `SymChar` and `SymEmpty` hold none, so the arity rules of FR-008, FR-010 and FR-011 are enforced by the type rather than checked at runtime.
- No exception is raised by any pure function. `Main` catches the I/O exceptions of reading and writing the *symbol tree* file and reports them on *stderr*.

## Purity and effect boundaries

Pure: `Genc3.Element`, `Genc3.Symbol`, `Genc3.Stream`, `Genc3.Syntax`, `Genc3.Parser`, `Genc3.Render`, `Genc3.Binary`, `Genc3.Cli` — every function declared in § Function signatures except `main`.

Effectful: `Main` alone. It reads *stdin* (IR-001), reads the *symbol tree* file where it exists and writes it back (IR-003, IR-004, IR-007), and writes *stdout* (IR-005) and *stderr* (IR-006). It contains no derivation, rendering, or parsing logic of its own; each stage of § Data flow that it owns is a call into a pure function. Deciding what the absence of the *symbol tree* file means is likewise not its own: it reports the absence as `Nothing` and `mTreeOfExisting` gives that absence its meaning.

The *memo table* is threaded explicitly through `outDerivation` as an argument and a result rather than held in mutable state, so *memoisation* (FR-004, FR-005) stays inside the pure part of *product*.

## Traceability

| Requirement | Realised by |
| --- | --- |
| FR-001 | `prsResult`, `sOverwritten` |
| FR-002 | `SymbolTree` |
| FR-003 | `synTerminalChar`, `synSequence`, `synNamedSequence`, `synSelection`, `synNamedSelection`, `synEmpty`, `synNamedEmpty` |
| FR-004 | `Memo`, `outDerivation`, `SynRef`, `Definitions`, `synReference`, `eDefsChecked`, `defsSyntax`, `nmsLeftRecursive` |
| FR-005 | `Memo`, `Outcome`, `Syntax`, `outDerivation`, `mOutAt`, `memInserted` |
| FR-006 | `Syntax`, `Symbol` |
| FR-007 | `Outcome`, `outDerivation` |
| FR-008 | `SymChar`, `outDerivation` |
| FR-009 | `SymSeq`, `outDerivation` |
| FR-010 | `SymSel`, `outDerivation` |
| FR-011 | `SymEmpty`, `outDerivation` |
| FR-012 | `outDerivation` |
| FR-013 | `ParserResult`, `prsResult`, `instance HumanRendering ParserResult` |
| FR-014 | `PrsError`, `OutNoMatch`, `ErrorTree`, `instance HumanRendering ParserResult` |
| FR-015 | `rpOfExtent`, `rpAt` |
| FR-016 | `ReadPos`, `rpOfExtent`, `rpAt`, `instance HumanRendering ReadPos` |
| FR-017 | `Syntax`, `Symbol`, `synNamedSequence`, `synNamedSelection`, `synNamedEmpty` |
| FR-018 | `HumanRendering`, `sRendering`, `instance HumanRendering SymbolTree`, `sSymbolRendering` |
| FR-019 | `sLineOfSymbol`, `sIndexOfSymbol`, `instance HumanRendering ReadPos` |
| FR-020 | `sElemKind` |
| FR-021 | `sElemId` |
| FR-022 | `instance HumanRendering SymbolTree`, `sLineOfSymbol` |
| FR-023 | `sElemId` |
| FR-024 | `Extent`, `rpOfExtent`, `extOfChar`, `sSymbolRendering` |
| FR-025 | `Extent`, `rpOfExtent`, `extOfChar`, `sSymbolRendering` |
| FR-026 | `Extent`, `rpOfExtent`, `extOfChar`, `sSymbolRendering` |
| FR-027 | `Guides`, `sSymbolRendering`, `sPrefixOfGuides` |
| FR-028 | `Guides`, `sPrefixOfGuides` |
| FR-029 | `SymSel`, `sIndexOfSymbol`, `outDerivation` |
| FR-030 | `bsOfTree`, `mTreeOfBinary` |
| FR-031 | `ErrorTree`, `ErrorKind`, `rpAt`, `instance HumanRendering ErrorTree` |
| FR-032 | `ErrorTree`, `outDerivation` |
| FR-033 | `ErrorTree`, `outDerivation` |
| FR-034 | `ErkUnconsumed`, `prsResult` |
| FR-035 | `instance HumanRendering ErrorTree` |
| FR-036 | `Symbol`, `Memo`, `outDerivation`, `sLineOfSymbol`, `mTreeOfRendering`, `bsOfTree`, `mTreeOfBinary` |
| FR-037 | `Reuse`, `memOfTree`, `prsResult` |
| FR-038 | `Reuse`, `memOfTree` |
| FR-039 | `Ground`, `Reuse`, `memOfTree` |
| FR-040 | `Reuses`, `instance HumanRendering Reuses`, `memOfTree` |
| FR-041 | `Reuses`, `instance HumanRendering Reuses` |
| FR-042 | `sLineOfReuse`, `sElemKind`, `sElemId` |
| FR-043 | `sLineOfReuse`, `Detail` |
| FR-044 | `sLineOfReuse`, `sGround`, `GrdShifted` |
| FR-045 | `sLineOfReuse`, `sGround`, `GrdBounded` |
| NFR-001 | `Stream`, `strmOfString`, `mChAt`, `nLengthOfStream`, `nmsLeftRecursive`, `prsResult`, `outDerivation`, `Memo` |
| NFR-002 | `memOfTree`, `prsResult` |
| NFR-003 | `memOfTree`, `prsResult`, `Memo`, `outDerivation`, `Symbol` |
| NFR-004 | `memOfTree`, `prsResult`, `Memo`, `outDerivation`, `Symbol` |
| IR-001 | `main` |
| IR-002 | `Options`, `Offset`, `Width`, `eOptions` |
| IR-003 | `Options`, `mTreeOfRendering`, `mTreeOfExisting` |
| IR-004 | `main`, `sRendering` |
| IR-005 | `main`, `sRendering` |
| IR-006 | `main`, `sRendering` |
| IR-007 | `mTreeOfExisting`, `mTreeOfExistingBinary`, `main` |
| IR-008 | `Options`, `eOptions` |
| IR-009 | `bsOfTree`, `main` |
| IR-010 | `sRendering`, `main` |
| IR-011 | `mTreeOfBinary`, `mTreeOfExistingBinary`, `mTreeOfExisting`, `main` |
| IR-012 | `Options`, `eOptions` |
| IR-013 | `Detail`, `mDetailOfKeyword`, `Options`, `eOptions` |
| IR-014 | `DetBasic`, `Options`, `eOptions` |
| IR-015 | `Reuses`, `instance HumanRendering Reuses`, `prsResult`, `main` |
| IR-016 | `Options`, `main` |
| IR-017 | `ParserResult`, `main` |
| C-001 | Implementation Phase (Stack project already in place) |
| C-002 | `Genc3.Syntax`, `defsSyntax` |

## Open questions

- none -
