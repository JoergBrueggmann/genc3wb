# Coding conventions

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [AD01] | 01_01_management.md | Management requirements |
| [AD02] | 02_01_requirements.md | Requirements guideline |
| [AD03] | 02_02_design.md | Design guideline |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | The software product being developed; formally defined by the Requirements specification (folder '03_Requirements'). |

## 1. General formatting

- **Line length** — a source code line is at most 80 characters; a comment line is at most 160 characters.
- **Indentation** — four spaces per level; tabs are never used.

## 2. Naming conventions

- Binding names (functions, local variables, arguments) use lowerCamelCase.
- Where a binding's result has a specific type, or is restricted to a type class, the name is prefixed with a short, lowercase hint for that type, followed by a capitalised, speaking name — e.g. `nRowPos` (a `Num`), `isHead` (a `Bool`).
- A speaking name denotes what the binding yields — its result — rather than the input it consumes or the subject it acts upon. A noun naming that result is the preferred form (`count`, `matrix`, `cursor`, `sRendering`); an adjective is not used (`readable`). A verb is acceptable where it names the result idiomatically and a noun form would be contrived (`take`, `replicate`), and is the natural form where the result is itself an action to be performed, such as an `IO` action. Where a noun and a verb read equally well, the noun is chosen.
- Where the result is most clearly identified by what it was derived from, the name takes the form `<result>Of<source>` — e.g. `rpOfExtent`, a `ReadPos` derived from an `Extent`; `chOfCode`, a `Char` derived from a character code.
- A name fixed by the language or by an interface being implemented — `main`, and the method names of a type class being instantiated — is exempt from the two rules above.
- An abbreviation inside a name keeps camel case rather than being fully capitalised: only its first letter is capitalised, whatever its length — e.g. `sSqlQuery`, `nAsciiCode`, `sXmlHeader`, and not `sSQLQuery`, `nASCIICode`, `sXMLHeader`.

### 2.1 Standard type-hint prefixes

| Type | Prefix | Example |
| --- | --- | --- |
| `Num a` | `n` | `nRowPos` |
| `Int` | `n` / `nj` | `njExp` |
| `Bool` | `is` / `do` / `has` | `isHead`, `hasBeenUpdated` |
| `[a]` | `l` | `lErr` |
| `Char` | `ch` | `chRead` |
| `String` | `s` / `lch` | `sStream` |
| `Maybe a` | `m` | `mCh` |
| `IO a` | `io` / `ios` | `io`, `ios` |

Common module-qualified prefixes: `Ex` (`Control.Exception`), `BS` (`ByteString`), `Chr` (`Char`), `Lst` (`List`), `Dir` (`Directory`).

### 2.2 Custom prefixes

A project-specific type or type class may define its own prefix instead of, or alongside, the standard ones above. A custom prefix is declared once, in that type's or type class's Haddock comment, as a `prefix: <value>` bullet, using the same lowerCamelCase convention as the standard prefixes.

## 3. Module structure

Every module is laid out in this order:

1. **Module description** — a Haddock comment giving Description, Copyright, License, Maintainer, Stability, and Portability. The copyright notice names the author in full, followed by the range of years in which the module's work began and was last revised, in the form `(c) <author>, <first year>-<latest year>`. The latest year is brought to the current year whenever the module is revised — the notice is therefore maintained per file, deliberately, in exchange for each file stating the period its content covers. For example:

   ```haskell
   {-|
   Description : <one-line summary of what the module provides>
   Copyright   : (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
   License     : All rights reserved
   Maintainer  : info@joerg-brueggmann.de
   Stability   : experimental
   Portability : POSIX
   -}
   ```

   The `License` field asserts the reservation of rights; the project's 'LICENSE' file states it in full and carries the same notice in its own legal form — `Copyright <first year>-<latest year> <author>`, with `All rights reserved.` beneath it. That form abbreviates the author's name where the module header spells it in full; the two forms are intentionally distinct and neither is to be aligned to the other.

2. **GHC extensions and options** — one `{-# LANGUAGE ... #-}` pragma per line, e.g.:

   ```haskell
   {-# LANGUAGE OverloadedStrings #-}
   {-# LANGUAGE GADTs #-}
   {-# LANGUAGE StandaloneDeriving #-}
   ```

3. **Module declaration and export list.**

4. **Sections** — the module's declarations, grouped and separated by a comment banner, in this order: type classes, then data types (each together with its instances), then their associated functions. A section banner is set off by two blank lines from the text above it, so sections stay visually distinct when scanning the file.

## 4. Declarations

### 4.1 Class and instance declarations

A type class's Haddock comment documents the capability it provides; each method's type signature is annotated per §5 below. An instance declaration is documented per constructor or case it handles, so each case's behaviour — and, where relevant, its output — is individually clear rather than left to be inferred from the code.

### 4.2 Data type declarations

Every data type declaration is preceded by a single-line comment giving its name (e.g. `-- Token …`), followed by a Haddock comment with a short description and, where useful, a bulleted list of notable properties.

- **Sum type** — each constructor gets its own Haddock comment explaining what it represents, e.g.:

  ```haskell
  -- EOLMode …
  {-| The line-ending convention a stream uses. -}
  data EOLMode
      = WindowsEOL -- ^ lines end in @\r\n@
      | UnixEOL    -- ^ lines end in @\n@
  ```

- **Product type** — fields are positional, not record syntax (see §4.3), each documented individually, e.g.:

  ```haskell
  -- Geoposition …
  {-| A point on Earth's surface. -}
  data Geoposition = Geoposition
      Double -- ^ latitude
      Double -- ^ longitude
  ```

- **Mixed type** — a sum of alternatives that themselves carry positional fields; both the alternatives and their fields are documented as above, e.g.:

  ```haskell
  -- ComposableOutput …
  data ComposableOutput
      = CmpValid   InputStream SymTree ErrorTree -- ^ parsed without error
      | CmpInvalid InputStream ErrorTree          -- ^ parsed with error
  ```

### 4.3 Record syntax

A record field name is prefixed with `r` (e.g. `rBits`, `rsStream`, `rposStream`) to avoid shadowing an unrelated binding of the same name at a pattern match.

Record syntax is not used on a sum type with more than one constructor: a field accessor generated for one constructor becomes a partial function when applied to a value built with another constructor, failing at runtime rather than being caught by the compiler. A sum type instead uses positional fields (§4.2), together with explicit accessor or smart-constructor functions where field access is genuinely needed.

## 5. Function declarations

- A function's Haddock comment describes what the function provides — its contract — not a narration of how it is implemented.
- The comment states what each parameter and the result represent, cross-referencing other declared names in single quotes where that helps.
- All naming rules from §2 apply to function names and their parameters.

```haskell
-- functionName
{-| What the function provides.

* notable property or precondition
* another notable property
-}
functionName
    :: Type1  -- ^ description of this parameter
    -> Type2  -- ^ description of this parameter
    -> Result -- ^ description of the result
```


## 6. EBNF comment of a carried syntax

Where a configured syntax is carried in the *product*'s own source rather than stated in a configuration file — the seed by which a configuration file's own notation is read — the module carrying it holds one comment stating that syntax in EBNF form. The comment sits at the head of the module's Functions section, before the first composing function, so that the whole syntax is readable in one place before its parts. For example:

```
    Syntax in EBNF:
        DecimalNumber = NonZeroDecimalDigit, DecimalDigits;
        DecimalDigit = NonZeroDecimalDigit | '0';
        DecimalDigits = ( DecimalDigits, DecimalDigit ) | ();
        NonZeroDecimalDigit = '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9';
        Sign = '+' | '-' | ();
```

The comment is the rendering *product* makes of that syntax, copied in unaltered, so keeping it current is a mechanical step and no notation rule is applied by hand. It is brought up to date in the same change that alters the syntax; one differing in any character from that rendering is treated as a defect of that change.
