# Specification

Product name: *genc³wb*

This specification follows the structure defined in '02_01_requirements.md'.

## Applicable documents

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | 01_01_management.md | Management requirements |
| [2] | 02_01_requirements.md | Requirements guideline |

## Terms

| Term | Context | Definition |
| --- | --- | --- |
| *product* | | The software product being developed: the workbench *genc³wb*, which edits the inputs of *genc³*, runs it, and presents its results. |
| *compiler-compiler* | | The program *genc³*, which *product* runs. It reads a *compiler-compiler input file* and a *compiler input file* and writes its results to standard output, to standard error, and to *output files*. |
| *compiler-compiler input file* | | The file that configures the syntax and the generators of the *compiler-compiler*. |
| *compiler input file* | | The file that the *compiler-compiler* parses according to the syntax configured in the *compiler-compiler input file*. |
| *input group* | Main window (FR-001) | One of the two groups of the main window that edit a file: the group of the *compiler-compiler input file* and the group of the *compiler input file*. |
| *processing state* | Shown by the indicator of an *input group* (FR-017) | The combination of whether the named file is known to exist and whether the edited text differs from it, defined by FR-018 to FR-021. |
| *output page* | Output group (FR-027) | One page of the output group: the standard output page, the standard error page, or one *output file* page. |
| *output file* | | One of the up to nine files that the *compiler-compiler* writes and *product* presents, each identified by its number. |
| *detached window* | | A window that presents the output group, or one editor, outside the main window, and that shows the same content as the group it was detached from. |

## Scope

In scope: the user interface of *product* — the main window and its groups (FR-001 to FR-006), the editing of the *compiler-compiler input file* and of the *compiler input file* with their file selection and their automatic saving (FR-007 to FR-016), the indication of the *processing state* (FR-017 to FR-021), the selection of the *compiler-compiler* (FR-022 to FR-026), the presentation of its results on *output pages* with their navigation and their *output file* selection (FR-027 to FR-042), the *detached windows* (FR-043 to FR-047), the restoring of the paths between sessions (FR-048 to FR-049), and the interfaces by which the user, the file system and the *compiler-compiler* reach *product* (IR-001 to IR-014).

Out of scope: the behaviour of the *compiler-compiler* itself, which is specified separately; when *product* decides to run the *compiler-compiler* and how it passes its arguments, which is specified in a later version; and the persistence format of the settings *product* stores between sessions.

## Functional requirements

### Main window

FR-001 [Data]: The *product* shall present a main window composed of a group for the *compiler-compiler input file*, a group for the *compiler input file*, a group for the *compiler-compiler*, and a group for the output.

FR-002 [Data]: The main window shall carry a menu bar with a help menu.

FR-003 [Behavioural]: When the user activates the info entry of the help menu, the *product* shall present a modal info dialog.

FR-004 [Behavioural]: When the user closes the info dialog, the *product* shall return the focus to the main window.

FR-005 [Data]: The main window shall carry a status bar.

FR-006 [Behavioural]: When the focus moves from one widget of the main window to another, the *product* shall notify the widget that lost the focus and the widget that received it.

### Editing an input file

FR-007 [Data]: An *input group* shall be composed of a file name field, a file selector button, a code editor, and a *processing state* indicator.

FR-008 [Functional]: The code editor shall display the line number of each line of its text in an area at its left edge.

FR-009 [Behavioural]: When the code editor holds the focus, it shall highlight the line the text cursor is in.

FR-010 [Behavioural]: When the code editor loses the focus, it shall stop highlighting that line.

FR-011 [Behavioural]: When the user activates the file selector button of an *input group*, the *product* shall present a file selector dialog, with the caption and the file filter of that *input group*.

FR-012 [Behavioural]: When the user selects a file in that dialog, the *product* shall write the selected path into the file name field of that *input group*.

FR-013 [Behavioural]: When the file name field of an *input group* changes and the named file exists, the *product* shall load the content of that file into the code editor of that *input group*.

FR-014 [Behavioural]: When the file name field of an *input group* changes while the code editor holds unsaved changes, the *product* shall ask the user whether to save those changes before it loads the named file.

FR-015 [Behavioural]: When the file name field of an *input group* changes and the named file does not exist, the *product* shall leave the content of the code editor unchanged.

FR-016 [Behavioural]: When the user has not typed in a code editor for the configured pause, the *product* shall save the content of that code editor to the file named in that *input group*.

### Processing state

FR-017 [Data]: The indicator of an *input group* shall show the *processing state* of that *input group* as one of four distinct images.

FR-018 [Data]: The *processing state* shall be *valid file, text untouched* where the named file exists and the edited text does not differ from it.

FR-019 [Data]: The *processing state* shall be *valid file, text changed* where the named file exists and the edited text differs from it.

FR-020 [Data]: The *processing state* shall be *unknown file, text untouched* where it is not established that the named file exists and the text has not been edited since.

FR-021 [Data]: The *processing state* shall be *unknown file, text changed* where it is not established that the named file exists and the text has been edited since.

### Selecting the compiler-compiler

FR-022 [Data]: The group for the *compiler-compiler* shall be composed of a file name field, a file selector button, and an indicator.

FR-023 [Behavioural]: When the user activates the file selector button of that group, the *product* shall present a file selector dialog for an executable file.

FR-024 [Behavioural]: When the user selects a file in that dialog, the *product* shall write the selected path into the file name field of that group.

FR-025 [Functional]: The *product* shall determine whether the file named in that group is executable.

FR-026 [Behavioural]: When the file named in that group is not executable, or an *input group* is not in the *processing state* *valid file, text untouched*, the *product* shall indicate that it cannot run the *compiler-compiler*.

### Presenting the results

FR-027 [Data]: The output group shall present one *output page* at a time, out of a standard output page, a standard error page, and one page per enabled *output file*.

FR-028 [Data]: The standard output page shall be composed of a read-only code editor showing what the *compiler-compiler* wrote to standard output.

FR-029 [Data]: The standard error page shall be composed of a read-only code editor showing what the *compiler-compiler* wrote to standard error, and a read-only field showing its exit code.

FR-030 [Data]: An *output file* page shall be composed of an enabling check box, a file name field, a file selector button, and a read-only code editor showing the content of that *output file*.

FR-031 [Data]: The output group shall present at most nine *output file* pages, numbered 1 to 9.

FR-032 [Behavioural]: When the user activates the navigation button to the right, the *product* shall present the next *output page*.

FR-033 [Behavioural]: When the user activates the navigation button to the left, the *product* shall present the previous *output page*.

FR-034 [Behavioural]: When the presented *output page* is the last one, the *product* shall disable the navigation button to the right.

FR-035 [Behavioural]: When the presented *output page* is the first one, the *product* shall disable the navigation button to the left.

FR-036 [Data]: The output group shall show which *output page* of how many is presented.

FR-037 [Behavioural]: When the user clears the enabling check box of an *output file* page, the *product* shall stop presenting that *output file*.

FR-038 [Behavioural]: When the user activates the file selector button of an *output file* page, the *product* shall present a file selector dialog and write the selected path into the file name field of that page.

FR-039 [Behavioural]: When the file named on an *output file* page changes, the *product* shall show the content of the newly named file in the code editor of that page.

FR-040 [Behavioural]: When the *compiler-compiler* has run, the *product* shall replace the content of the standard output page, of the standard error page, and of every enabled *output file* page.

FR-041 [Behavioural]: When the *product* starts a run of the *compiler-compiler*, it shall clear the content of every *output page*.

FR-042 [Functional]: A code editor of an *output page* shall not accept text entered by the user.

### Detached windows

FR-043 [Behavioural]: When the user activates the detach button of the output group, the *product* shall present the output group in a *detached window*.

FR-044 [Behavioural]: When a *detached window* of the output group exists and the content of an *output page* changes, the *product* shall show the changed content in the main window and in every *detached window* of the output group.

FR-045 [Behavioural]: When the user activates the detach button of an *input group*, the *product* shall present the code editor, the file name field and the indicator of that *input group* in a *detached window*.

FR-046 [Behavioural]: When the user closes a *detached window*, the *product* shall stop showing content in it and shall leave the main window unchanged.

FR-047 [Behavioural]: When the *product* terminates, it shall close every *detached window*.

### Settings

FR-048 [Behavioural]: When the *product* starts, it shall restore the path of the *compiler-compiler input file*, the path of the *compiler input file*, the path of the *compiler-compiler*, and the path of every *output file*, as they were when it last terminated.

FR-049 [Behavioural]: When one of the paths of FR-048 changes, the *product* shall store it.

## Non-functional requirements

NFR-001 [Usability]: The *product* shall indicate the *processing state* of an *input group* within 200 milliseconds of the change that caused it.

NFR-002 [Performance]: The *product* shall load a *compiler input file* of at most 1 megabyte into a code editor within 1 second.

NFR-003 [Performance]: The *product* shall show the content of an *output file* of at most 1 megabyte within 1 second of the *compiler-compiler* having run.

NFR-004 [Portability]: The *product* shall run on Linux, MacOS (Apple silicon), and Microsoft Windows without a change of its source code.

## Interface requirements

### Graphical user interface

IR-001 (input): The *product* shall accept the text of a *compiler-compiler input file* and of a *compiler input file*, entered by the user in the code editor of the respective *input group*.

IR-002 (input): The *product* shall accept the path of a *compiler-compiler input file*, of a *compiler input file*, of the *compiler-compiler*, and of an *output file*, entered by the user in the respective file name field.

IR-003 (input): The *product* shall accept the selection of a file, made by the user in a file selector dialog.

IR-004 (input): The *product* shall accept the enabling and disabling of an *output file*, made by the user with the enabling check box of that *output file* page.

IR-005 (input): The *product* shall accept the navigation to the previous and to the next *output page*, made by the user with the navigation buttons.

IR-006 (output): The *product* shall present the *processing state* of each *input group* as an image.

IR-007 (output): The *product* shall present what the *compiler-compiler* wrote to standard output, what it wrote to standard error, and its exit code.

IR-008 (output): The *product* shall present the content of every enabled *output file*.

### File system

IR-009 (input): The *product* shall read a *compiler-compiler input file*, a *compiler input file*, and an *output file*, by the path named for it.

IR-010 (output): The *product* shall write a *compiler-compiler input file* and a *compiler input file*, by the path named for it.

IR-011 (input): The *product* shall read the paths it stored at its last termination.

IR-012 (output): The *product* shall write the paths it is to restore at its next start.

### Compiler-compiler

IR-013 (output): The *product* shall run the *compiler-compiler* as a separate process, by the path named for it.

IR-014 (input): The *product* shall accept the standard output, the standard error and the exit code of that process.

## Constraints

C-001: The *product* shall be implemented in C++ using the Qt framework.

C-002: The *product* shall present its user interface with the widgets of the Qt framework, rather than with widgets of another toolkit.

C-003: The *product* shall run the *compiler-compiler* as a separate process, rather than link it as a library.

C-004: The *product* shall keep the code editor of an *input group* independent of the *compiler-compiler*, so that a file can be edited while no *compiler-compiler* is named.

## Open questions

- none -
