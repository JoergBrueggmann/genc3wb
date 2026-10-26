# genc³wb

genc³wb is the workbench of genc³, the meta compiler-compiler and build system
of software 4sd. genc³ chains your abstractions into a build network: every
step from one abstraction to the next — a domain language to a model, a model
to code, a binary frame to text — is one readable rule file, a network file
chains the steps, and every edit is rebuilt only where it matters. genc³wb
shows that network as a graph while genc³ evaluates it, opens each step with
its rule file, inputs and outputs, marks diagnostics where they belong, and
shows the outputs of the chain after every accepted change — as text, hex or
bits.

What genc³ is, what is available today and what is planned:
[sw4sd.com](https://www.sw4sd.com/) and its [About page](https://sw4sd.com/about/).
The news post
[genc³ in 2026: the build network is here](https://sw4sd.com/2026/10/04/genc%c2%b3-in-2026-the-build-network-is-here/)
shows genc³wb with the example that ships with genc³: a small class language
normalised into a record form and rendered as two C++20 files that compile —
three abstractions, two rule files, one network. The documentation of genc³
and genc³wb, a quick start included, will be published on sw4sd.com and later
on genc3.com.

## Status

- The source of genc³wb is public under the GNU General Public License,
  version 3 ('LICENSE'). The build system `genc3d` it drives belongs to genc³,
  which is released internally and available to design partners, not as a
  public download. To try genc³ and genc³wb on your case, use the
  [contact form](https://sw4sd.com/contact/) of software 4sd.
- genc³ builds for macOS on Apple silicon today; builds for Linux and Windows
  are planned. genc³wb reaches `genc3d` and its nodes through Unix domain
  sockets, so that in this version the graph and the node windows are shown on
  Linux and macOS alone; without a build system a network file is edited, but
  no graph is shown.
- The notation of the rule files and the network files may still change before
  genc³ 1.0.
- Planned: integration into existing IDEs through the Language Server Protocol,
  and AI assistance for writing rule files.

## Features

- **The build network, live.** The compiler network editor edits the network
  file while `genc3d` evaluates it and shows the network as a graph: the nodes,
  their rule files, inputs and outputs, and the files that connect them. You
  see at every moment whether the network is invalid, valid or active, and you
  activate and deactivate it with a switch.
- **One window per step.** A node opens as a machine: its inputs, its outputs
  and, as its control, its rule file, the meta compiler DSL. Several nodes are
  open at once, and a file that several windows present is one document, so
  that an edit shows everywhere it matters and reaches every node that has it.
- **Edits travel as deltas.** Every edit reaches the node as a change, not as a
  file, after a short idle time the workbench tunes to the measured answer
  times. The outputs follow as the node holds them and are stored once the
  network accepts the change, so that the chain stays live while you type.
- **Diagnostics where they belong.** Errors, warnings and information are
  marked in the editor they concern, whoever caused them, with the message at
  hand and listed on a page of their own. A node downstream of an error points
  at the node it originates at, so that a fault is found where it was made.
- **Text, hex and bits.** Inputs and outputs are octets, shown and edited as
  UTF-8 text, as hexadecimal digits or as bits, so that binary data is as much
  at home as text, with selection, clipboard, undo and redo in every view.
- **Editor windows on any file of the network.** Any input or output opens in
  a window of its own, which keeps the nodes that have it running, so that a
  chain runs and its outputs follow without a node window open.
- **Started quickly.** A new network file comes from a template, and the rule
  files and inputs a node lacks are created on your word, with a rule that
  passes every character through, so that a first network runs before a rule
  is written. Tabs or spaces, as you prefer.
- **Automation, for tests and demonstrations.** Every session is recorded as
  you work, as a script in a notation genc³ itself parses, together with the
  files the network works on and your settings, so that what you did can be
  played back from the same state: to reproduce a problem exactly as it
  happened, to run a scenario as a test that verifies what the editors show,
  or to demonstrate the workbench step by step, at the pace you choose.
- **At home on the desktop.** A double-click on a gc3n-file opens it with the
  workbench, every network in a workbench of its own, and a lock file keeps two
  workbenches from editing one network. The network is closed with a click,
  and the settings travel with the installation.

## Built with

genc³wb is written in Rust. Its user interface is a Qt Quick application,
reached from Rust through Qt Bridge for Rust, the crate `qtbridge`; no C++ is
written. Building from source needs Rust 1.89 or later, Qt 6.10 or later with
Qt Quick and `qmake` on the path, a C++ toolchain matching that Qt, and
Graphviz, whose `dot` lays the graph out:

```bash
cargo build --release
```

`cargo test` runs the tests of the application logic; the integration tests
against the build system run with the path of `genc3d` in the environment
variable `GENC3D` and the option `--ignored`. The script 'make-dist.sh'
assembles the distribution folder with the executables of genc³.

NOTE: Portions of this project's code and documentation were developed with AI
assistance (Claude), under the author's direction, review, and authorisation.
