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
  file while `genc3d` evaluates it and shows the network it states as a graph:
  the nodes, their rule files, inputs and outputs, and the files that connect
  them. The mode of the network — invalid, valid or active — stands in the
  status bar and in the colours of the graph, and a switch activates and
  deactivates the network.
- **One window per step.** A double-click on a node opens it as a machine: the
  inputs on the left, the outputs on the right, and the rule file, the meta
  compiler DSL of the node, below as its control. Several nodes are open at
  once, and a file that several windows present is one document: an edit in
  one window shows in the others and reaches every node that has the file.
- **Edits travel as deltas.** Every edit of a rule file or of an input is
  transmitted to the node after a short idle time, which the workbench sets
  from the measured answer times. The outputs are shown as the node holds them
  and are stored after every change the network accepts without an error.
- **Diagnostics where they belong.** Errors, warnings and information are
  marked in the editor they concern, whoever caused them — an edit in the
  window or a node upstream that produced an input anew — with the message as
  a tooltip, and are listed on a page of their own. A node downstream of an
  error points at the node it originates at.
- **Text, hex and bits.** Inputs and outputs are held as octets and shown as
  UTF-8 text, as hexadecimal digits or as binary digits. Inputs are edited in
  all three, digit by digit, with selection, clipboard, undo and redo, and
  inserting or overwriting typing. A diagnostic of a binary document is marked
  by its bit offset.
- **Editor windows on any file of the network.** A double-click on an input or
  an output of the graph opens it in a window of its own. The window holds the
  nodes that have its file, so that the chain runs and the outputs follow with
  no node window open.
- **Started quickly.** The `+` button creates a network file from a template
  of two nodes. When you open a node or activate the network while a rule file
  or an input does not exist yet, a dialog names the files and creates them on
  your word, folders included: the rule file from a template that reads every
  UTF-8 character and renders the symbol tree, an input empty, so that a first
  network runs before a rule is written. The dialog can be switched off for
  good, in itself or in the settings.
- **Tabs or spaces.** The Tab key inserts a tab character, or spaces up to the
  next tab stop when you prefer them, a setting beside the tab size.
- **At home on the desktop.** A double-click on a gc3n-file opens it with the
  workbench, every network in a workbench of its own, and a lock file beside
  the network file names the workbench that holds it. The settings lie beside
  the executable, so that the distribution is one folder.

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
