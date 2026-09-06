# Coding conventions

This document defines the conventions for *product*'s source code (folders 'src', 'app' and 'test', output of phase 5 *Implementation and Test* — see '01_01_management.md' § Dependency order), in addition to the general rules of '01_01_management.md' (§ Document rules). It is tailored to *product* being built in C++ with the STL and Qt (see '00_agents.md').

The conventions are re-engineered from the predecessor of *product* ('00_agents.md' § Predecessor), which is written to them consistently. Where this document departs from the predecessor, it says so and why.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | 01_01_management.md | Management requirements |
| [2] | 02_01_requirements.md | Requirements guideline |
| [3] | 02_02_design.md | Design guideline |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | The software product being developed; formally defined by the Requirements specification (folder '03_Requirements'). |
| *type hint* | Prefix of a name (§2.2) | A short lowercase prefix naming the type of what a binding holds, placed before the binding's speaking name. |
| *speaking name* | Part of a name (§2) | The part of a name that says what the binding yields, after any *type hint*. |

## 1. General formatting

- **Line length** — a source code line is at most 120 characters; a comment line is at most 160 characters. The predecessor was written to 80, which its aligned declarations and Qt's long type names exceed in 105 of 2937 lines; 120 fits the style instead of fighting it.
- **Indentation** — four spaces per level; tabs are never used.
- **Braces** — a class or function *definition* opens its brace on its own line; a statement — `if`, `else`, `for`, `while`, `switch` — opens its brace on the same line:

  ```cpp
  void Gc3TimerWatchdog::on_timerTick()
  {
      if ( m_bEnabled ) {
          m_nTicksNoKeepAlive++;
      }
  }
  ```

- **Spacing** — a condition carries one space inside its parentheses, `if ( m_bEnabled )`; a call does not, `start(m_nMilliSecondsPerTick)`.
- **Alignment** — consecutive member declarations may be aligned in columns where that makes the group readable. Alignment is never a reason to exceed the line length.

## 2. Naming conventions

- A file name is all lowercase without separator, and carries the prefix of its role (§2.1): 'ctrlinpfilehandling.h', 'mdlccsettings.cpp'.
- A class name is UpperCamelCase and mirrors its file name, prefix included: 'ctrlinpfilehandling.h' declares `CtrlInpFileHandling`; 'gc3timerwatchdog.h' declares `Gc3TimerWatchdog`.
- A function, a parameter and a local variable use lowerCamelCase; a member variable is prefixed `m_`.
- Where a binding holds a value of a specific type, its name is prefixed with the *type hint* for that type (§2.2), followed by a capitalised *speaking name* — `m_strCInpFilePath`, `nFileNumber`, `pwdgtParent`, `m_bSetupValid`.
- A *speaking name* denotes what the binding yields — its result — rather than the input it consumes or the subject it acts upon. A noun naming that result is the preferred form (`count`, `cursor`, `strRendering`); an adjective is not used (`readable`). A verb is acceptable where it names the result idiomatically and a noun form would be contrived, and is the natural form where the result is itself an action to be performed. Where a noun and a verb read equally well, the noun is chosen.
- Where the result is most clearly identified by what it was derived from, the name takes the form `<result>Of<source>` — `strPathOfUrl`, a `QString` derived from a URL.
- A name fixed by the language, by Qt, or by an interface being implemented — `main`, an overridden virtual member function, a Qt slot connected by name — is exempt from the two rules above.
- An abbreviation inside a name keeps camel case rather than being fully capitalised: only its first letter is capitalised, whatever its length — `strSqlQuery`, `nAsciiCode`, `strXmlHeader`, and not `strSQLQuery`, `nASCIICode`, `strXMLHeader`.
- A Qt slot that answers an event is named `on_<event>`: `on_selectFile`, `on_fileNameChanged`, `on_timerTick`.

### 2.1 File name prefixes

| Prefix | Holds |
| --- | --- |
| `mdl` | a model: data and its persistence, free of user interface |
| `ctrl` | a controller: the handling that connects a model to widgets |
| `gwb` | a window or dialog of the application |
| `gc3` | a reusable widget, usable outside the application |

A file that belongs to none of these roles carries no prefix — 'main.cpp'.

### 2.2 Standard type-hint prefixes

| Type | Prefix | Example |
| --- | --- | --- |
| `bool` | `b` / `is` / `has` / `do` | `m_bEnabled`, `isFileValidAndSaved` |
| integral and floating types | `n` | `nFileNumber`, `m_nTicksThreshold` |
| `QString` | `str` | `m_strCInpFilePath` |
| `QChar`, `char` | `ch` | `chRead` |
| an enumeration value | `e` | `m_eState` |
| a pointer | `p` | `m_pCCSettings` |
| a pointer to a widget | `pwdgt` | `m_pwdgtParent` |
| a pointer to a member function | `fp` | `fpMember` |
| `QMap`, `std::map` | `map` | `m_mapOutFilePath` |
| `QVector`, `QList`, `std::vector` | `vec` | `vecFilePath` |
| a template type parameter | `T` | `TClass` |

A prefix names what the binding holds, not how it is passed: a `const QString&` parameter is `strFileName`, not `rstrFileName`.

### 2.3 Custom prefixes

A type declared by *product* may define its own prefix instead of, or alongside, the standard ones above. A custom prefix is declared once, in that type's documentation comment (§5), as a `@par prefix` paragraph, using the same lowercase convention as the standard prefixes.

### 2.4 Namespaces

- Every declaration of *product* is placed in the namespace `genc3wb::<component>`, where `<component>` is the component the file belongs to, as the design names it ('02_02_design.md' § Architecture overview).
- A component name inside a namespace is all lowercase without separator, as a file name is: `genc3wb::inputfile`, not `genc3wb::inputFile` and not `genc3wb::input_file`.
- The namespace is opened in a header after the includes and closed before the `#endif` of the include guard, and in a source file after the includes.
- A `using namespace` directive is not written at file scope of a header, so that a file including it inherits no name it did not ask for. In a source file a `using` declaration for a single name is permitted.
- The predecessor carries no namespace. A declaration carried over from it is placed in the namespace of its component.

## 3. File structure

### 3.1 File header

Every source and header file opens with a documentation comment:

```cpp
/**
 * @file      ctrlinpfilehandling.h
 * @brief     <one-line summary of what the file provides>
 * @copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
 * @author    Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
 */
```

The copyright notice names the author in full, followed by the years `2021-<current year>`: 2021 is the year in which the work of *product* began, and the second year is brought to the current year whenever the file is revised. Every file of *product* carries the notice in this one form, so that it reads the same everywhere and can be checked mechanically.

The file header asserts no licence. The licence of *product* is stated once, in the file 'LICENSE', and is neither repeated nor summarised in a file header: a header naming a licence would have to be corrected in every file of *product* whenever that licence changed, and a header disagreeing with 'LICENSE' would leave it unclear which of the two holds.

A file carried over from elsewhere under a licence of its own keeps that licence's notice unchanged and does not receive the copyright notice above. Its `@file` and `@brief` are written beneath the foreign notice, so that the file is documented as a file of *product* without its licence being altered. 'gc3codeeditorwidget.h' of the predecessor, carried over from an example of the Qt Company under a BSD licence, is such a file.

### 3.2 Header file

A header file is laid out in this order:

1. The file header (§3.1).
2. The include guard, whose macro is the file name in upper case with its dot replaced by an underscore: 'ctrlinpfilehandling.h' guards with `CTRLINPFILEHANDLING_H`. The `#endif` repeats the macro as a trailing comment.
3. The includes, in three groups separated by a blank line: the headers of *product* first, then the Qt headers, then the standard library headers.
4. The namespace of the component (§2.4), containing the class declaration (§4).

The guard is derived from the file name and follows it when the file is renamed. In the predecessor 12 of 13 headers guard with the name the file carried before its prefix was introduced — `TIMERWATCHDOG_H` in 'gc3timerwatchdog.h' — which is the defect this rule prevents.

```cpp
#ifndef GC3TIMERWATCHDOG_H
#define GC3TIMERWATCHDOG_H

#include "gc3lineedit.h"

#include <QTimer>

#include <iostream>

namespace genc3wb::widget
{

class Gc3TimerWatchdog : public QTimer
{
    // ...
};

}

#endif // GC3TIMERWATCHDOG_H
```

### 3.3 Source file

A source file is laid out in this order:

1. The file header (§3.1).
2. Its own header, included first and alone, so that the header is proven to compile on its own.
3. The remaining includes, grouped as in §3.2.
4. The definitions, in the order in which their declarations appear in the header.

## 4. Declarations

### 4.1 Class declarations

- A class derived from `QObject` opens its body with `Q_OBJECT`.
- The body is divided into access sections, each labelled with a trailing comment naming what it holds: `public: // methods`, `protected: // attributes`, `protected: // internal methods`.
- Within a section, related declarations are grouped and each group is introduced by a single-line comment: `// initialisation`, `// change detection`, `// state`.
- A member function that a derived class may replace is declared `virtual`, the destructor included.
- Qt signals are declared in a `signals:` section, slots in a `public slots:` or `protected slots:` section, both after the ordinary sections.

### 4.2 Enumerations

An enumeration is declared as an `enum class`, as the design guideline requires ('02_02_design.md' § Type declarations):

```cpp
/** @brief The file the settings apply to. */
enum class SettingsType
{
    CompilerInput,          ///< the input file of the compiler
    CompilerCompilerInput   ///< the input file of the compiler-compiler
};
```

The predecessor declares `typedef enum Type_E { MdlCCStgs_CInp, ... } Type_T;`, whose enumerators need a manual prefix because they share the enclosing scope. A scoped enumeration makes the prefix unnecessary and the conversion unchecked-to-checked; the predecessor's enumerations are converted when their code is carried over.

### 4.3 Member variables

- Every member variable is prefixed `m_` and carries its *type hint* (§2.2).
- Every member variable is documented, either by a trailing `///<` comment or by the group comment that introduces it (§4.1).
- A member variable is `protected` or `private` unless the class exists to carry data without invariants.

## 5. Documentation comments

Every declaration that forms a public interface — a class, an enumeration, a member function, a free function — carries a Doxygen comment. The predecessor documents nothing this way; this is the one convention re-engineering does not find in it, and it is required because the design guideline traces requirements to declarations ('02_02_design.md' § Traceability) and the test guideline validates against the documented contract ('02_03_test.md' § Validation checklist).

- The comment describes what the declaration provides — its contract — not a narration of how it is implemented.
- `@brief` gives one sentence. `@details` adds properties and preconditions, one per line.
- `@param` states what each parameter represents, `@return` what the result represents.
- A requirement realised by the declaration is named as the design guideline requires, in a comment directly above it.

```cpp
// realises FR-012
/**
 * @brief   Loads the file into the code editor.
 * @details * The file is read as UTF-8.
 *          * Where the file does not exist, the editor is left unchanged.
 * @param   strFileName                 the path of the file to load
 * @param   bSaveIfItCouldNotBeLoaded   whether to write the editor's content back where the file could not be read
 * @return  whether the file was loaded
 */
virtual bool load(const QString& strFileName, bool bSaveIfItCouldNotBeLoaded = false);
```

A declaration that is not part of a public interface is documented where its purpose is not evident from its name and signature.
