# genc3wb

The workbench of the compiler-compiler genc³: it edits the two input files of
genc³, runs it, and presents what it wrote to standard output, to standard error
and to its output files.

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

The QML files of the user interface are checked with the linter of Qt:

```bash
qmllint -I src/qml src/qml/*.qml
```

### Where the settings are stored

genc³wb restores the paths of the input files, of the compiler-compiler and of
the output files at its start, from `genc3wb/settings.txt` in the configuration
directory of the user: `~/.config` on Linux, `~/Library/Application Support` on
macOS, `%APPDATA%` on Windows. The file is plain text, one `key = value` line
per path.
