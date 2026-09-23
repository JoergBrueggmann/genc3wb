# genc3wb

The language workbench of the meta compiler-compiler genc³. Its main window is the compiler
network editor: it edits a compiler network file (gc3n-file), lets a build system
such as `genc3d` evaluate it while it is edited, and shows the network it states
as a graph. A double-click on a node of the graph opens the node window, which
shows the node as the network file states it: its meta compiler DSL, its inputs
and its outputs. The build system serves that one node while its files are
edited: every change is transmitted to it, its diagnostics are shown at the
editor they concern, and its outputs are stored and presented after every
change it accepts without an error.

NOTE: Portions of this project's code and documentation were developed with AI
assistance (Claude), under the author's direction, review, and authorisation.

genc³wb is written in Rust. Its user interface is a Qt Quick application,
reached from Rust through Qt Bridge for Rust, the crate `qtbridge`. No C++ is
written; a C++ toolchain is needed only because the crate builds its own
bridging code against Qt.

## Preconditions

- Rust, stable, 1.87 or later, with cargo, from <https://rustup.rs>. A Rust
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

- A build system that conforms to genc³api 0.9.0.0 and offers the description
  mode, which is `genc3d` of genc³ 0.18.0.0 or later. It is named in the
  compiler network editor; without it a network file is edited, but no graph is
  shown. genc³wb reaches it through a Unix domain socket, so that in this
  version the graph is shown on Linux and macOS alone.

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

Three tests run the build system and `dot` and are therefore left out of that
run. They are run with the path of `genc3d` in the environment variable
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

The file is YAML and holds the idle time and the long idle time in seconds,
the path of the network file and the path of the build system. The node window
names nothing of its own: the meta compiler DSL, the inputs and the outputs of a
node are taken from the network file when the node is opened. Where the file
does not exist, the idle time is 2 seconds, the long idle time 16 seconds and
the build system `./genc3/bin/genc3d`. The two times are set in the dialog of
the settings menu.

### The node window

A double-click on a meta compiler-compiler in the graph opens it. The build
system named in the compiler network editor is started for that one node, in
the directory of the network file, and shut down when another node is opened,
when the node window closes and when the workbench terminates. An edit of the
meta compiler DSL or of an input is saved after the idle time and transmitted to
the node, which answers with its diagnostics; they are shown below the editor.
After every change the node accepts without an error, its outputs are stored and
shown on the output pages.

An input that another node produces is editable all the same. As soon as it is
edited, an exclamation mark appears beside its file name, whose tooltip names
the producing node: the edit is temporary. When the node is shut down, the file
is written back to what it held when the node was started, so that the
producing node finds what it stored.
