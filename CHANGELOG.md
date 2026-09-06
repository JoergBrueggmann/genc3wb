# Changelog for `genc³wb`

Copyright 2021-2026 Jörg Karl-Heinz Walter Brüggmann

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to a four-part version number.


## Unreleased

### [0.3.0.0] - YYYY-MM-DD

**_Editing_of_the_input_files_**

- Implement the component 'genc3wb::inputfile': drive one input group with its file name field, its file selector button, its code editor and its processing state indicator, as the design declares it.
- Present a file selector dialog on the file selector button, with the caption and the file filter of that input group, and write the selected path into the file name field.
- Load the named file into the code editor where it exists, leave the editor unchanged where it does not, and ask the user whether to save before discarding unsaved changes.
- Save the content of the code editor to the named file after the configured pause in typing, driven by the watchdog.
- Determine the processing state from whether the named file exists and whether the edited text differs from it, and show it in the indicator.
- Announce that the file or its content changed, so that the group of the compiler-compiler can tell whether it may run.
- Connect the two input groups of the main window to the settings, so that the paths are restored at the start and stored at the end.
- Cover the component with a test group and its test cases, reading and writing files in a directory of the run.

### [0.4.0.0] - YYYY-MM-DD

**_Change_to_Rust_with_Qt_Bridging_Technology_**

- Change agent, management, guidelines, requirements, design, test and code to transform everything to Rust and its ecosystem using Qt's bridging technology.


## Released

### [0.2.0.1] - 2026-09-06

**_Reporting_of_a_failed_deployment_**

- Take the compiler and the make of the Qt installation, rather than the first of that name on the path, so that the product is linked with the toolchain Qt is built with. Another compiler of the same name links against a different C runtime, and the link then fails on symbols of the entry point of Qt.
- Report the compiler that is used, next to qmake and make.
- End the deployment where the build fails, naming the build as the reason, instead of reporting a missing test executable further on.
- Refuse the deployment where the output directory cannot be replaced, naming the application to close, instead of failing with the message of the remove command.
- Report at the end the name, the size and the time of the executable produced, so that a bundle left over from an earlier run is recognised as such.

### [0.2.0.0] - 2026-09-06

**_Main_window_of_the_workbench_**

- Implement the component 'genc3wb::mainwindow': compose the main window from the group of the compiler-compiler input file, the group of the compiler input file, the group of the compiler-compiler and the output group, built in code from the widget tree of the design.
- Lay out each input group with its file name field, its file selector button, its code editor and its processing state indicator, from the widgets of 'genc3wb::widget'.
- Carry a menu bar with a help menu, present a modal info dialog from that menu, and carry a status bar.
- Notify the widget that lost the focus and the widget that received it, so that the code editor highlights its current line only while it holds the focus.
- Initialise the resources of *product*, which a static library does not initialise by itself and without which the indicator draws no image.
- Declare 'InputGroup', 'initResources' and the accessors of the main window in the design, which the implementation needs and the design did not declare.
- Cover the main window with a test group of ten test cases, and assert that the four images of the indicator resolve.

### [0.1.0.1] - 2026-09-06

**_Portable_build_of_the_application_**

- Provide the script 'deploy.sh', which builds the application, runs the test suite, and gathers the application with every library it needs into one folder that can be copied to a machine carrying neither Qt nor a compiler.
- Fail the deployment where the test suite fails, so that a failing build is never packaged.
- Verify the result by reading the imports of every binary of the folder, and require each imported library to lie in the folder or to belong to the operating system.
- Carry the software renderer and the D3D compiler along, without which the platform plugin is found but cannot be initialised.
- Pin the plugins of the folder with a 'qt.conf', so that a Qt installation of the target machine cannot divert the application to the plugins of another build.
- Exclude the folder of the portable build from version control.

### [0.1.0.0] - 2026-09-06

**_Implementation_of_the_foundation_**

- Set up the Qt project of *product* with qmake: a subdirs project over 'src', 'app' and 'test', 'src' building the components as a static library that 'app' and 'test' link, and C++23 as the standard the design assumes.
- Implement the component 'genc3wb::widget' as the design declares it: the line edit, the plain text edit with its watermark, the code editor with its line number area, the label of several states, the indicator of the processing state with its four images, and the watchdog of the typing pause.
- Implement the component 'genc3wb::settings' as the design declares it: the paths of the two input files, of the compiler-compiler and of the nine output files, restored at construction and stored at destruction.
- Carry the widget 'Gc3CodeEditorWidget' over from the predecessor, keeping the licence notice of the Qt Company that holds for it.
- Open an empty main window in the application.
- Set up the test suite on the framework 'neatest' with the assertion macro 'TEST_ASSERT', and cover the two components with 2 test groups and 17 test cases.
- Run the test suite in a working directory of its own, so that what one run stores leaves nothing behind for the next.
- Declare 'setOutFilePath' and 'strCCExecFilePath' in the design, which the component 'genc3wb::settings' needs and the design did not declare.
- Define in the test guideline the entry point that runs the root of the test suite and yields its result as the exit code.
- Exclude the build output and the stored settings from version control.

### [0.0.0.15] - 2026-09-06

**_Settlement_of_the_open_items_**

- Define the namespace 'genc3wb::<component>' in the coding conventions, name a component in it as a file name is named, and place a declaration carried over from the predecessor in the namespace of its component.
- Assert no licence in the file header: state the licence of *product* once, in 'LICENSE', so that no file header can disagree with it.
- Carry the years '2021-<current year>' in the copyright notice of every file of *product*, and set the first year to 2021 in 'LICENSE'.
- Keep the licence notice of a file carried over from elsewhere unchanged, and write its '@file' and '@brief' beneath that notice.
- Admit a document of the same phase in the applicable documents where it is cited, and keep a document of a later phase excluded.
- Write no changelog entry for a change that alters no meaning, and enter a correction that does change meaning.
- Correct the namespace examples of the design guideline and of the test guideline, which still name the product of the predecessor.

### [0.0.0.14] - 2026-09-06

**_Design_of_the_user_interface_**

- Replace the inherited design of *genc³* by a design of *genc³wb* that realises the requirements FR-001 to FR-049 of the specification.
- Re-engineer the components of the design from the predecessor: the model of the settings, the controllers of the input files, of the compiler-compiler and of the output, the reusable widgets, and the windows.
- Extend the 'Design guideline' to demand a user interface design: a widget tree per window, with the class, the parent and the requirement of each widget, and a layout sketch per window.
- Follow the design guideline: decompose into components with acyclic dependency, declare the types and the functions in C++ without bodies, show every class in a class diagram, and state the data flow, the error handling strategy, the side-effect boundaries and the traceability to the requirements.
- Decompose *product* into the seven components 'widget', 'settings', 'inputfile', 'output', 'runner', 'editorwindow' and 'mainwindow', in that dependency order.

### [0.0.0.13] - 2026-09-06

**_Requirements_regarding_UI_**

- Re-engineer the requirements for the user interface of *product* from the predecessor, and replace the inherited specification of *genc³* by a specification of *genc³wb*.
- Specify the main window and its four groups, the editing of the two input files with their file selection and their automatic saving, the indication of the processing state, the selection of the compiler-compiler, the presentation of its results on output pages, the detached windows, and the restoring of the paths between sessions, as FR-001 to FR-049.
- Specify the graphical user interface, the file system and the compiler-compiler as the three interfaces of *product*, as IR-001 to IR-014.
- Constrain *product* to C++ with the Qt framework, and to running the compiler-compiler as a separate process, as C-001 to C-004.

### [0.0.0.12] - 2026-09-06

**_Coding_guideline_**

- Consolidate the 'Coding guideline' and re-engineer 'old_src' to define the coding of the *product*: resolve its references to the management requirements, follow the reference convention, and replace the Haskell conventions by the C++ and Qt conventions the predecessor is written to.
- Remove the section on the EBNF comment of a carried syntax.
- Define the file name prefixes 'mdl', 'ctrl', 'gwb' and 'gc3' for the role a file holds, and mirror the file name in the class it declares.
- Define the type-hint prefixes for C++ and Qt, and keep the member prefix 'm_' of the predecessor.
- Set the line length of code to 120 characters, which the aligned declarations and the Qt type names of the predecessor need.
- Codify the brace style of the predecessor: a definition opens its brace on its own line, a statement on the same line.
- Derive the include guard from the file name, which 12 of the 13 headers of the predecessor no longer do.
- Require a Doxygen comment on every declaration of a public interface, which the predecessor carries nowhere.
- Declare an enumeration as 'enum class', as the design guideline requires, instead of the typedef of the predecessor.

### [0.0.0.11] - 2026-09-06

**_Imperative_changelog_entries_**

- Write every entry of a work item in the imperative, as a commit message is written.
- Read an entry of 'Unreleased' as what is to be done, and the same entry of 'Released' as what was done.
- Rewrite the entries of the released versions 0.0.0.1 to 0.0.0.10 in the imperative.

### [0.0.0.10] - 2026-09-06

**_Test_guideline_**

- Consolidate the 'Test guideline': define the test suite of *product* on the macro framework 'neatest' of 'old_tst', with the three levels root, test group and test case, and the file layout of the folder 'test'.
- Extend the framework by the assertion macro 'TEST_ASSERT', which names the expression, the file and the line where an expectation broke.
- Exclude the folder 'old_tst' from version control, and record it in '00_agents.md' as reference material next to 'old_src'.

### [0.0.0.9] - 2026-09-05

**_Design_guideline_**

- Consolidate the 'Design guideline' as the 'Requirements guideline' was consolidated in version 0.0.0.8: resolve its references to the management requirements, drop the working folder and the finalisation checklist, and follow the reference convention and the document rules in the design template.
- Make the component the unit of decomposition, named by its C++ namespace and by the files that make it up, and keep acyclic dependency as a rule of the guideline, justified by readability instead of by the compiler.
- Give type declarations and function declarations in C++ syntax without bodies, and declare an abstraction as an abstract base class or as a concept, stating the choice.
- Name in the error handling strategy the C++ means of representing expected failure, and require an exception that crosses a component boundary to be declared with that boundary.
- Require a section 'Class diagram' in which every declared class appears with its relationships to the other classes, given as Mermaid source.
- Rename the section 'Purity and effect boundaries' to 'Side-effect boundaries', stating which parts of *product* are free of side effects and which perform I/O, with 'const' as the marker in the declaration.

### [0.0.0.8] - 2026-09-05

**_Requirements_guideline_**

- Define the document rules for applicable documents and for terms in '01_Management/01_01_management.md'.
- Consolidate the 'Requirements guideline': resolve its references to the management requirements, drop the working folder and the finalisation checklist, and follow the reference convention and the document rules in the specification template.
- Name the framework of *product* in the constraint example.
- Exclude the folder 'old_src', the predecessor of *product*, from version control, and record it in '00_agents.md' as reference material.

### [0.0.0.7] - 2026-09-05

**_Attribution_in_the_readme_instead_of_the_commits_**

- Carry no co-author trailer in a commit message, and state the assistance of the agent once in 'README.md'.
- Define the form of a commit message.

### [0.0.0.6] - 2026-09-05

**_Code_phrases_of_the_trigger_points_**

- Enter the code phrases 'start change cycle', 'agreed' and 'abort change cycle' for the trigger points TP-1 to TP-3.
- Define that a code phrase fires its trigger point wherever it appears in the client's prompt.

### [0.0.0.5] - 2026-09-05

**_Deletion_of_the_change_branch_**

- Delete the change branch after the push is verified, and keep it where the change cycle is aborted.
- Delete the change branches of the versions 0.0.0.1 to 0.0.0.4.

### [0.0.0.4] - 2026-09-05

**_Explicit_adding_of_all_files_**

- Add the whole working tree with 'git add .' before committing, and verify afterwards that no untracked file is left.
- List adding as a step of its own in the release and integration step, and verify the push against the remote.

### [0.0.0.3] - 2026-09-05

**_Separation_of_released_versions_**

- Hold unreleased work items in section 'Unreleased' and released versions in section 'Released', the most recent first.
- Move a work item to 'Released' when the change cycle releases it.

### [0.0.0.2] - 2026-09-05

**_Version_control_of_all_project_files_**

- Put every file of *product* under version control, and cover the whole working tree with the commit of the change cycle, not only the files of the work item.
- Report at the gate the files the work item did not change, before they are committed.
- Place the guideline, requirement and design documents under version control unchanged, to be adapted later.
- Name *product*, its year and its author in the licence notice.

### [0.0.0.1] - 2026-09-05

**_Finalisation_of_agent_and_management_definition_**

- Write the initial version of '00_agents.md': general information, agent role, persona, decision authority and reporting.
- Write the initial version of '01_Management/01_01_management.md': applicable documents, dependency order, change cycle and the client's code phrases.
- Take in the change cycle the next unreleased work item of this file, identify the files to change, go along the folder dependencies, and end on the client's agreement with release, commit, pull, merge and push.
- Define phases as a dependency structure only, through which the client does not step.
- List as applicable only the input documents of a document, and list '00_agents.md' nowhere, since it is implicitly applicable to all of them.
- Define the four-part version number, determined and maintained by the change cycle.
- Print with the code phrases 'print for review' and 'print changes for review' the relevant input and output of the current change, in full respectively as the changes reported by git.
- Leave the code phrases of the trigger points TP-1 to TP-3 to be supplied by the client.
