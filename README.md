# genc3wb

The language workbench of the meta compiler-compiler genc³. Its main window is the compiler
network editor: it edits a compiler network file (gc3n-file), lets a build system
such as `genc3d` evaluate it while it is edited, and shows the network it states
as a graph. A double-click on a node of the graph opens a node window, one per
node and several at once, which shows the node as the network file states it:
its meta compiler DSL, its inputs and its outputs. The build system runs the
whole network live while its files
are edited: every change is transmitted to the node, the diagnostics of the
node are shown at the editor they concern, whoever caused them, its outputs are
presented as the node holds them, and the outputs of every node are stored
after every change the node accepts without an error, while no node of the
network has an error and the network is active.

The network has a mode, which the status bar of the main window shows: it is
invalid while the network file cannot be read as a network, valid while it can
and no node runs, and active while every node runs. The build system activates
a valid network when it starts; where a node's meta compiler DSL is invalid, the
activation fails, the network stays valid and the error names the node. The
switch `ina | act` beside the build system shows whether the network is active
and switches it: `act` activates it, once the files are corrected, and the
build system refuses while the network is invalid; `ina` deactivates it, which
stores the outputs and ends every node. The switch is greyed while no mode is
known and while a request is under way, which the status bar tells. A node is
opened while the network is valid or active: the workbench holds the node, so
that the build system starts and serves it in a valid network, and the nodes of
the open node windows run on when the network is deactivated; closing a node
window releases its node, which ends where no other client holds it. Every
node window closes when the network becomes invalid, for example when the
network file is broken, since no node runs then. The status line of the node
window names the mode of the node: invalid, waiting for its meta compiler DSL,
valid or active. A file that several node windows present is held once: an edit
in one window shows in the others and reaches every node that has the file. A meta compiler DSL that another node produces is shown
read-only with the content the node holds, as an input of a producer is. The switch `ina | act`
and the switch `txt | hex | bin` of the editors carry a tooltip that spells
their positions out. The graph tells the mode as well: while the network is
valid or active, a meta compiler-compiler is a subdued blue, a proxy node a
subdued yellow, an input that no node produces a subdued orange, an output
that no node consumes a subdued green, and the file of a connection white;
while the network is invalid, or no mode is known, the same hues are nearly
grey and dimmed. Every meta compiler-compiler of a valid or active network,
and every such input and output in every mode, carries the small text
`double click here` in its top left corner: a double-click on an input opens
an editor window on the file, which edits it as the node windows do and
reaches every node that has it, and a double-click on an output opens a
read-only editor window showing what the producing node holds while a node
window serves it, and the file otherwise. The meta compiler DSL of a meta compiler-compiler is a vertex of its
own, an input of the node, so that a node carries its name alone; a proxy node
carries its tool chain below its name. The graph runs from left to right: the
inputs enter a node at its left side, the outputs leave it at its right side,
and the meta compiler DSL enters it from below, standing below its node unless
another node produces it.

An input and an output are held as their octets, whatever they are. The code
editor of each has a switch of three positions: `txt` shows the octets read as
UTF-8, with the replacement character U+FFFD for every octet that is no
character, `hex` shows them as hexadecimal digits, and `bin` as binary digits,
as many octets per row as the width of the editor holds: 16, 32 or 64 in `hex`,
4, 8, 16 or 32 in `bin`. The switch starts at `txt` for a text and at
`hex` for other octets. An edit in `txt` replaces the octets by the UTF-8
encoding of the text. An input that is no valid UTF-8 is served as a binary
document, and a diagnostic is marked in `hex` and `bin` by its bit offset.

An input is edited in `hex` and `bin` digit by digit, the nibble in `hex` and
the bit in `bin`. Backspace removes the digit before the cursor and Delete the
one at it, the digits behind moving toward the start. The digits that pad the
last octet are placeholders, shown as `_` and worth 0, until they are
overwritten or the long idle time expires. Digits are selected with Shift and
the movement keys or by dragging, copied, cut and pasted as text, and edits are
undone and redone. The outputs stay read-only.

Typing inserts or overwrites, in `txt`, `hex` and `bin` alike. The typing mode
is toggled by the insert key and by the menu `Edit`, and is shown at the bottom
of every window with an editor. The insert key is fn + Return (Enter) on macOS
and Insert elsewhere; `Settings > Insert key...` records another one, which is
stored in `genc3wb.yaml` as `insert_key`.

NOTE: Portions of this project's code and documentation were developed with AI
assistance (Claude), under the author's direction, review, and authorisation.

genc³wb is written in Rust. Its user interface is a Qt Quick application,
reached from Rust through Qt Bridge for Rust, the crate `qtbridge`. No C++ is
written; a C++ toolchain is needed only because the crate builds its own
bridging code against Qt.

## Preconditions

- Rust, stable, 1.89 or later, with cargo, from <https://rustup.rs>. A Rust
  installed earlier is brought up to date with `rustup update stable`.

- Qt 6.10 or later for the desktop, with the modules `qtbase` and
  `qtdeclarative`; the latter holds Qt Quick, Qt Quick Controls, Qt Quick
  Dialogs and the tool `qmllint`. `qmake` of that Qt has to be found on the
  path, since the crate `qtbridge` locates Qt through it. On Debian and Ubuntu
  the tool is named `qmake6`; set `QMAKE=qmake6` there.

- A C++ compiler and a linker that match the Qt installation: the command line
  tools of Xcode on macOS, MSVC 2022 on Windows, `g++` on Linux.

- Graphviz, whose program `dot` lays out the graph of a compiler network. It
  is searched on the path, and then in `/opt/local/bin`, `/opt/homebrew/bin`,
  `/usr/local/bin` and `/usr/bin`. The graph is rendered as a PNG image, so
  that the module `qtsvg` of Qt is not needed.

- A build system that conforms to genc³api 0.19.0.0, which is `genc3d` of
  genc³ 0.32.1.0 or later: genc³wb has it activate the network and run every
  node of it, holds and releases the nodes of the node windows, watches the
  modes and the states of the nodes, and reads what a node holds of its inputs
  and outputs. It is named in the compiler network editor; without it a network
  file is edited, but no graph is shown. genc³wb reaches it and its nodes
  through Unix domain sockets, so that in this version the graph is shown on
  Linux and macOS alone.

Qt Bridge for Rust supports Linux x86_64, Windows x64 and macOS arm64, the
last as experimental.

### What to obtain, per operating system

|  | Windows | macOS | Linux |
| --- | --- | --- | --- |
| Rust | rustup, from the browser | `curl https://sh.rustup.rs -sSf \| sh` | `curl https://sh.rustup.rs -sSf \| sh` |
| Compiler | Visual Studio 2022 Build Tools | the command line tools of Xcode | `g++` |
| Qt | Qt 6 desktop, the MSVC build, with Qt Quick | Qt 6 for macOS with Qt Quick | the development packages of Qt 6 base and declarative, with the private headers |

**Windows.** Install the Build Tools for Visual Studio 2022 with the C++
workload, and Qt with the Qt Online Installer from
<https://www.qt.io/download-qt-installer>: tick, under the Qt 6 version you
want, the MSVC 2022 64-bit entry, which carries Qt Quick. Then put the `bin`
directory of that Qt on the path, the version being the one you installed:

```powershell
$env:PATH = "C:\Qt\6.11.2\msvc2022_64\bin;" + $env:PATH
```

**macOS.** The command line tools, and a Qt without a Qt account, installed
with `aqtinstall`. Run these in Terminal, in any directory:

```bash
xcode-select --install
python3 -m pip install --user aqtinstall
python3 -m aqt install-qt mac desktop 6.11.2 clang_64 \
    --outputdir ~/Qt6.11.2 --archives qtbase qtdeclarative qttools
export PATH="$HOME/Qt6.11.2/6.11.2/macos/bin:$PATH"
```

Write the `export` line into `~/.zprofile` to keep it beyond the session. The Qt Online Installer serves as well, and asks for a Qt account;
tick Qt Quick with it.

**Linux.** The compiler, the Qt of the distribution with its private headers,
and the path to its libraries. Run the lines of your distribution, in any
directory:

```bash
sudo apt install build-essential qt6-base-dev qt6-base-private-dev qt6-declarative-dev qt6-declarative-private-dev   # Debian, Ubuntu
export QMAKE=qmake6
```

```bash
sudo dnf install gcc-c++ qt6-qtbase-devel qt6-qtbase-private-devel qt6-qtdeclarative-devel qt6-qtdeclarative-private-devel   # Fedora
```

The executables carry the run path of the Qt they were built against, which
the build script reads from `qmake`, so that no `LD_LIBRARY_PATH` is needed.

## Getting started

### Installation from source code

Fetch the source and enter it:

```bash
git clone https://github.com/JoergBrueggmann/genc3wb.git
cd genc3wb
```

Build it, with `qmake` on the path:

```bash
cargo build --release
```

The first build compiles the bridging code of `qtbridge` against your Qt and
takes a few minutes; every later build takes seconds. The executable is left in
`target/release/genc3wb` (`genc3wb.exe` on Windows). It holds the whole user
interface; nothing but the Qt libraries is needed beside it.

Start it, in the directory the source was cloned into:

```bash
cargo run --release
```

or start the executable itself:

```bash
target/release/genc3wb
```

The executable carries the run path of the Qt it was built against, so that it
finds the Qt frameworks on macOS and the Qt libraries on Linux without an
environment variable. On Windows the Qt `bin` directory is on the path instead.

What genc³wb writes to its own standard output — the increments by which its
code editors report how their text changed — is read in the terminal it was
started from.

### Running the test suite

The tests of the application logic run without Qt being started:

```bash
cargo test
```

The exit code is 0 where every test passed. The run is stored under `reports`
with

```bash
mkdir -p reports && cargo test 2>&1 | tee reports/test.txt
```

Eleven tests run the build system, with the nodes of a network, and `dot` and
are therefore left out of that run. They are run with the path of `genc3d` in the environment variable
`GENC3D`:

```bash
GENC3D=/path/to/genc3d cargo test --test build_system -- --ignored
```

The QML files of the user interface are checked with the linter of Qt:

```bash
qmllint -I src/qml src/qml/*.qml
```

### Making a distribution

The script 'make-dist.sh' assembles the folder `dist`: the executable of the
workbench under `bin`, the scripts `start` and `start.bat`, this description,
the licence and the work items of the package, and, under `genc3`, a copy of the
distribution of genc³ with its build system `genc3d`. What is distributed
therefore holds the workbench and the build system it drives.

**The distribution is started by a double-click on `start`, or on `start.bat` on
Windows.** Each script changes to the folder it lies in, so that the workbench
finds its settings file and the build system beside itself.

The script first runs 'make-dist.sh' of genc³, so that the copy is the
distribution that was just built and verified there, and then builds the
workbench with `cargo build --release`. The project folder of genc³ is given as
the first argument, or in the environment variable `GENC3`:

```bash
GENC3=/path/to/genc3 ./make-dist.sh
```

The exit code is 0 where the folder was assembled, the executable of the
workbench and the script `start` are executable, and the copied `genc3d` runs.
In the distribution the build system is named `./genc3/bin/genc3d`, beside the
workbench, which is the path genc³wb uses where it has none stored.

### Where the settings are stored

genc³wb reads the file `genc3wb.yaml` of its working directory at its start and
writes it whenever a value changes. Started by a double-click on `start`, the
working directory is the root of the distribution, so the file lies there.

The file is YAML and holds the idle time and the long idle time in seconds
with one decimal, whether the times are set automatically, the path of the
network file, the path of the build system and the insert key. The node window names nothing of
its own: the meta compiler DSL, the inputs and the outputs of a node are taken
from the network file when the node is opened. Where the file does not exist,
the idle time is 2 seconds, the long idle time 16 seconds, the automatic setting
on, and the build system `./genc3/bin/genc3d`.

The two times are set in the dialog of the settings menu, in steps of 0.2
seconds; the idle time is at least 0.2 seconds and the long idle time is
greater than the idle time, and the dialog says so where a value violates it.
With the automatic setting on, the workbench measures how long the node or the
build system takes to answer a change — up to the storing of the outputs — and
sets the idle time to twice that, rounded up to 0.2 seconds, and the long idle
time to eight times the idle time, after every answered change.

### One workbench per network and per settings file

A genc³wb holds a lock file beside the network file it has open, named like
the network file with `.lock` appended, e.g. `network.gc3n.lock`, and one
beside its settings file, `genc3wb.yaml.lock`, which the first genc³wb started
in a folder takes for its whole run. A lock file names its holder: the process
number, the host and the time it was taken. A second genc³wb on the same Mac
does not open a network another one holds: it keeps the network it had, or
opens none at its start. Started in the same folder, it works with the
settings it read but stores none of its changes. Either refusal is told by a
dialog and in the status bar of the main window, naming the holder.

The lock is the advisory lock of the operating system, which is released when
its holder terminates in whatever way, so that a lock file left behind by a
crash blocks nothing. In this version the locks are made for MacOS on a local
disk, and they bind genc³wb alone: a `genc3d` started in a terminal or a text
editor is not kept from the files.

### The node windows

A double-click on a meta compiler-compiler in the graph opens it in a node
window of its own; a second double-click raises that window. The window shows
the node as a machine: the inputs on the left, the outputs on the right, the
gear in the middle, and the meta compiler DSL below as its control. The build
system named in the compiler network editor runs every node of the network
file while the network is active, and the nodes of the open node windows while
it is valid: opening a node holds it, so that the build system starts it where
it does not run, and keeps it running through a deactivation. The node window
attaches to the running node: it connects to the service socket of the node
and starts no process; closing the window closes the connection and releases
the node, which the build system ends in a valid network where no other
client holds it, and leaves running in an active network. While the node is
not served, the output pages show the files of the outputs. An edit of the
meta compiler DSL or of an input is saved after the idle time and transmitted
to the node, and to every other node served that has the file, which the
windows of those nodes show as one document. The node tells its diagnostics, of its meta compiler
DSL and of every input, whenever they change, whoever caused the change: an
edit in the window, or a node upstream that produced one of the inputs anew.
Each is marked in its editor by a wavy line under the faulty place — red for
an error, orange for a warning, blue for information — whose tooltip shows the
message, and all of them are listed on the last page of the outputs,
'Diagnostics'. The output pages show what the node holds of its outputs, and
follow it as it changes. After every change the node accepts without an error,
the build system stores the outputs of every node, as long as no node of the
network has an error; while one has, the store is held back until none has.
Where the outputs could not be stored, the output pages carry an exclamation
mark beside the file name, whose tooltip names the failure; the failure is
listed on the page 'Diagnostics' as well, which is presented then. The mark is
gone with the next store that succeeds. The diagnostics of the network file are
marked in its editor the same way.

An input that another node produces is not edited in the node window: its
editor is read-only and shows what the node holds of it, which the producing
node overwrites whenever it produces it anew.

Where a node upstream has an error, the page 'Diagnostics' of every node
downstream of it carries an exclamation mark for it, whose tooltip names the
node the errors originate at; a click on it opens that node, where its
diagnostics are shown in full.
