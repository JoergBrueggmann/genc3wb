# Changelog for `genc³wb`

Copyright 2021-2026 Jörg Karl-Heinz Walter Brüggmann

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to a four-part version number.


## Unreleased

### [0.0.0.12] - YYYY-MM-DD

**_Coding_guideline_**

- Consolidate the 'Coding guideline' and re-engineer 'old_src' to define the coding of the *product*.


## Released

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
