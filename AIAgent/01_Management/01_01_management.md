# Management requirements

This document describes the management of *product*'s development process — which input shall lead to which output — and the management of the documents produced along the way.

## Applicable documents

### Rules

A document of *product* lists under its applicable documents only those documents that are its input according to section 'Definition and Folder dependencies'. A document that takes it as input is not listed, because the dependency runs in the opposite direction.

The file '00_agents.md' is not listed in the applicable documents of any document. According to section 'Definition and Folder dependencies' it is the input of every phase, and it is therefore implicitly applicable to every document of *product*.

### Documents applicable to this document

According to section 'Definition and Folder dependencies', the input of phase 1 *Management Definition* is '00_agents.md' and the client's prompt. By the rules above '00_agents.md' is implicitly applicable and is not listed, and the documents of the phases 2 to 5 take this document as input and are not listed either. What remains listed is the document that the change cycle maintains.

| Ref. | Document file name | Title |
| --- | --- | --- |
| [1] | CHANGELOG.md | Changelog for `genc³wb` |

## Phases

If the input is ambiguous or incomplete, the agent interviews the client with structured questions before writing any output.

### Overview

#### Definition and Folder dependencies

The scheme below describes which folder content will determine output in which file and folder, respectively.

prompt --Agent Definition --> 00_agents.md

00_agents.md + prompt --Management Definition--> 01_Management

01_Management + 00_agents.md + prompt --Guidelines Definition--> 02_Guidelines

02_Guidelines + 01_Management + 00_agents.md + prompt --Requirements Definition--> 03_Requirements

03_Requirements + 02_Guidelines + 01_Management + 00_agents.md + prompt --Design Definition--> 04_Design

04_Design + 03_Requirements + 02_Guidelines + 01_Management + 00_agents.md + prompt --Implementation and Test--> src / app / test

#### Dependency order

The phases are ordered as listed below. This order is the *Dependency order* referred to throughout this document. A phase is identified by its number or by its name.

| No. | Phase | Output path | Input phases |
| --- | --- | --- | --- |
| 0 | Agent Definition | `00_agents.md` | — |
| 1 | Management Definition | `01_Management` | 0 |
| 2 | Guidelines Definition | `02_Guidelines` | 0, 1 |
| 3 | Requirements Definition | `03_Requirements` | 0, 1, 2 |
| 4 | Design Definition | `04_Design` | 0, 1, 2, 3 |
| 5 | Implementation and Test | `src` / `app` / `test` | 0, 1, 2, 3, 4 |

The phases are a dependency structure only. The client does not step through them: no phase is announced, made active or finalised, and the client is not asked which phase a change belongs to. The change cycle below identifies the changed phases itself and goes along the dependencies on its own.

A change to a phase is propagated to every phase that has it among its input phases, in ascending order of the number.

### Change cycle

A change to *product* is carried out in one *change cycle*. The input of a change cycle is the next unreleased work item of `CHANGELOG.md` [1]; the cycle ends with that work item released and pushed. The agent drives the cycle without asking the client to manage its steps, and stops at the single gate defined below.

#### Version number

The version number of *product* has four parts:

    <major>.<minor>.<patch>.<build>

Exactly one part is incremented per released work item: the highest-ranking part that applies to what the work item changed. When a part is incremented, every part below it is set to 0.

| Part | Rank | Incremented where the work item |
| --- | --- | --- |
| major | highest | changes *product* incompatibly with the version before |
| minor | | adds functionality compatible with the version before |
| patch | | corrects a fault without changing functionality |
| build | lowest | changes no functionality of *product*, for example its definition documents only |

The agent determines the part from what the work item changed in fact, and reports it. Where the determination is not unambiguous, the agent asks the client.

As long as phase 5 *Implementation and Test* has no output, no work item can change functionality of *product*, and every work item increments the build part.

#### Changelog structure

`CHANGELOG.md` [1] holds the work items in two sections.

| Section | Holds | Order |
| --- | --- | --- |
| `## Unreleased` | the work items that are not released; their date is `YYYY-MM-DD` | the order in which they are to be worked on |
| `## Released` | the versions that are released; their date is the date of their release | the most recent version first |

A work item stays in `## Unreleased` for the whole change cycle. The release of step 9 moves it, with its entries, from `## Unreleased` to the top of `## Released`.

#### Version control

Every file of *product* belongs under version control. The change cycle leaves no file of the project untracked.

The commit of step 9 therefore covers the whole working tree, not only the files that the work item named. A file that the work item did not change is committed as it stands.

To leave no file out, the agent adds the whole working tree to the index with

    git add .

before it commits, and it verifies afterwards that git reports no untracked file left.

Before the gate the agent determines which files of the working tree are not yet under version control, and which carry changes it did not make itself. It reports them at the gate together with what it changed, so that the client sees the whole content of the commit before agreeing to it.

#### Steps

**Step 1 — Work item.**
The agent takes the next unreleased work item: the first version heading in section `## Unreleased` of `CHANGELOG.md` [1], together with the entries listed under it. The client's prompt may narrow the work item to a part of it, or extend it; an extension is written into the work item.

**Step 2 — Identification.**
The agent identifies which phases and which files of *product* the work item changes, and reports them.

**Step 3 — Branch.**
The agent creates one branch for the change cycle, from `main`, named

    change/<version>-<short topic>

for example `change/0.0.0.1-agent-and-management-definition`. The whole change cycle is carried out on this one branch, and the branch exists only for the duration of the change cycle.

**Step 4 — Change.**
The agent changes the identified files, on the client's prompts, and reports what it changed and why. It does not pause for confirmation before each individual edit.

**Step 5 — Dependency propagation.**
The agent goes along the dependencies of section 'Definition and Folder dependencies': for every phase it has changed, it adapts each phase that takes that phase as input, in the Dependency order, and reports each adaptation. A phase adapted this way is itself a changed phase, so the step repeats until no further phase is affected.

**Step 6 — Test.**
The agent runs the build and the tests, as far as they exist, and reports the result. A failing build or test returns the cycle to step 4.

**Step 7 — Changelog.**
The agent brings the entries of the work item in `CHANGELOG.md` [1] in line with what was changed in fact, including what the dependency propagation reached. It then checks the version number of the work item against the rules of section 'Version number' and corrects it where what was changed in fact requires another part to be incremented. The date of the version heading stays `YYYY-MM-DD`.

**Step 8 — Agreement gate (G).**
The agent stops, reports everything changed in the steps 4 to 7, and asks gate question **G**. It reports in addition the files that the commit will contain although the work item did not change them, according to section 'Version control'. It carries out none of step 9 before the client agrees.
An objection returns the cycle to step 4, or to step 2 where the identification itself was wrong.

**Step 9 — Release and integration.**
On the client's agreement the agent carries out the following, in this order and without further questions:

1. **release** — replace `YYYY-MM-DD` in the version heading of the work item in `CHANGELOG.md` [1] by the current date, and move the work item to the top of section `## Released`, according to section 'Changelog structure',
2. **add** — add the whole working tree to the index with `git add .`, according to section 'Version control',
3. **commit** — commit the index on the change branch,
4. **pull** — update `main` from its remote,
5. **merge** — merge the change branch into `main`,
6. **push** — push `main` to its remote, and verify that the remote holds what was pushed,
7. **delete** — delete the change branch.

Where the pull brings in changes that conflict with the change, the agent stops, reports the conflict, and returns the cycle to step 4.

The change branch is deleted only after the push has been verified, so that its content is on the remote before the branch is removed. A change cycle that is aborted keeps its branch, as section 'Client's code phrases' defines for TP-3. The change cycle ends with the deleted branch.

#### Gate

At the gate the agent stops, reports what it has done, and asks the single question defined for it. It proceeds only on the client's agreement.

| Gate | Position | Question |
| --- | --- | --- |
| G | Step 8 | "Work item *&lt;version&gt;* is complete: *&lt;summary of everything changed&gt;*. Do you agree, and shall I release, commit, pull, merge and push it?" |

The agreement given at the gate authorises the whole of step 9, the push included.

### Client's code phrases

A *trigger point* of the change cycle is fired either by a code phrase the client prompts or by a question the agent asks — the same mechanism from opposite ends. Where the client prompts a code phrase, it fires that trigger point and settles what the agent would otherwise have determined or proposed.

The code phrases are supplied by the client. Until a phrase is entered below, the corresponding trigger point can be fired by the agent's question only.

A code phrase fires its trigger point wherever it appears in the client's prompt. The client may embed it in a sentence, as in 'yes, agreed'.

The table below covers the trigger points of the change cycle only. Code phrases that do not belong to the change cycle are defined in '00_agents.md', for example the phrases 'print for review' and 'print changes for review'.

| Ref. | Trigger point | Code phrase | Effect when fired |
| --- | --- | --- | --- |
| TP-1 | Start change cycle | **start change cycle** | Starts a change cycle at step 1. A phrase that names a work item settles which one is taken. |
| TP-2 | Agree | **agreed** | Fires gate G and thereby authorises step 9. |
| TP-3 | Abort change cycle | **abort change cycle** | Ends the change cycle without release and without merge. The change branch is kept, `main` is left untouched. |
