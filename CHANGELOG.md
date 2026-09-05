# Changelog for `genc³wb`

Copyright 2021-2026 Jörg Karl-Heinz Walter Brüggmann

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to a four-part version number.


## Unreleased


## Released

### [0.0.0.8] - 2026-09-05

**_Requirements_guideline_**

- The document rules for applicable documents and for terms are defined in '01_Management/01_01_management.md'.
- The 'Requirements guideline' is consolidated: its references to the management requirements resolve, the working folder and the finalisation checklist are dropped, and the reference convention and the specification template follow the document rules.
- The constraint example names the framework of *product*.
- The folder 'old_src', the predecessor of *product*, is excluded from version control and recorded in '00_agents.md' as reference material.

### [0.0.0.7] - 2026-09-05

**_Attribution_in_the_readme_instead_of_the_commits_**

- Commit messages carry no co-author trailer; the assistance of the agent is stated once in 'README.md'.
- The form of a commit message is defined.

### [0.0.0.6] - 2026-09-05

**_Code_phrases_of_the_trigger_points_**

- The code phrases 'start change cycle', 'agreed' and 'abort change cycle' fire the trigger points TP-1 to TP-3.
- A code phrase fires its trigger point wherever it appears in the client's prompt.

### [0.0.0.5] - 2026-09-05

**_Deletion_of_the_change_branch_**

- The change cycle deletes the change branch after the push is verified; an aborted change cycle keeps it.
- The change branches of the versions 0.0.0.1 to 0.0.0.4 are deleted.

### [0.0.0.4] - 2026-09-05

**_Explicit_adding_of_all_files_**

- The change cycle adds the whole working tree with 'git add .' before it commits, and verifies afterwards that no untracked file is left.
- The release and integration step lists adding as a step of its own, and the push is verified against the remote.

    ### [0.0.0.3] - 2026-09-05

**_Separation_of_released_versions_**

- The changelog holds unreleased work items in section 'Unreleased' and released versions in section 'Released', the most recent first.
- The change cycle moves a work item to 'Released' when it releases it.

### [0.0.0.2] - 2026-09-05

**_Version_control_of_all_project_files_**

- Every file of *product* is under version control; the commit of the change cycle covers the whole working tree, not only the files of the work item.
- Files the work item did not change are reported at the gate before they are committed.
- The guideline, requirement and design documents are placed under version control unchanged, to be adapted later.
- The licence notice names *product*, its year and its author.

### [0.0.0.1] - 2026-09-05

**_Finalisation_of_agent_and_management_definition_**

- Initial version of '00_agents.md': general information, agent role, persona, decision authority and reporting.
- Initial version of '01_Management/01_01_management.md': applicable documents, dependency order, change cycle and the client's code phrases.
- The change cycle takes the next unreleased work item of this file, identifies the files to change, goes along the folder dependencies, and ends on the client's agreement with release, commit, pull, merge and push.
- Phases are a dependency structure only; the client does not step through them.
- A document lists as applicable only its input documents; '00_agents.md' is implicitly applicable to all of them and is listed nowhere.
- The four-part version number is defined; the change cycle determines and maintains it.
- The code phrases 'print for review' and 'print changes for review' print the relevant input and output of the current change, in full respectively as the changes reported by git.
- The code phrases of the trigger points TP-1 to TP-3 are still to be supplied by the client.
