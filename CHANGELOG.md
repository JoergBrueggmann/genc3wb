# Changelog for `genc³wb`

Copyright 2021-2026 Jörg Karl-Heinz Walter Brüggmann

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to a four-part version number.


## Unreleased


## Released

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
