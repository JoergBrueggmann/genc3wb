# genc3wb

The Integrated Development Environment for genc³ build environment.

NOTE: Portions of this project's code and documentation were developed with AI assistance (Claude), under the author's direction, review, and authorisation.

## Preconditions

- You have to hold a shell that runs `bash`. `deploy.sh` is a bash script; on
  Windows it is run from the shell of MSYS2 or of Git for Windows, and not from
  the command processor.

- You have to install Qt 6 for the desktop, so that `qmake` — or `qmake6` — is
  found on the path. Where you would rather name the installation than put it
  on the path, set `QMAKE` to the path of `qmake` itself, which `deploy.sh`
  reads in preference to the path.

- You have to install a C++ compiler that knows the C++23 standard, which the
  project asks for with `CONFIG += c++2b`, and a `make`. Which ones differ per
  platform, and `deploy.sh` names the one it misses:

  - Windows: the MinGW component of the Qt installation. `deploy.sh` takes the
    compiler and the make from Qt itself rather than from the path, because Qt
    on Windows is linked against the C runtime of the toolchain it ships with,
    and another compiler of the same name yields a link that fails on the
    symbols of the entry point of Qt.
  - macOS: the command line tools of Xcode, which carry `clang++`.
  - Linux: the `g++` and the `make` of the distribution.

- You have to install the deployment tool of Qt for your platform, which
  `deploy.sh` uses to carry the Qt libraries into the result: `windeployqt` on
  Windows, `macdeployqt` on macOS. Qt ships none for Linux, where `deploy.sh`
  collects the libraries itself.

- These are asked for by no step and `deploy.sh` says where one is absent, yet
  the result is the better for them:

  - `patchelf` or `chrpath` on Linux, without which the run paths of the
    binaries still name the machine that built them, and the application loads
    the Qt of that machine where it finds one.
  - `objdump` on Windows, `ldd` on Linux, `otool` and `lipo` on macOS, with
    which `deploy.sh` verifies that the result carries every library it imports
    and reports the architecture it was built for.

### What to obtain, per operating system

|  | Windows | macOS | Linux |
| --- | --- | --- | --- |
| Shell | MSYS2, or Git for Windows | `zsh`, already present | `bash`, already present |
| Compiler and make | the MinGW component of Qt | the command line tools of Xcode | `g++` and `make` |
| Qt | Qt 6 desktop, the MinGW build | Qt 6 for macOS | the development package of Qt 6 base |
| Deployment tool | `windeployqt`, which comes with Qt | `macdeployqt`, which comes with the Qt tools | none; `deploy.sh` collects the libraries itself |
| Worth having | — | — | `patchelf` or `chrpath` |

**Windows.** Two installations, both from the browser:

- MSYS2, from <https://www.msys2.org>, or Git for Windows, from
  <https://gitforwindows.org>. `deploy.sh` is run from the shell either of them
  installs, and not from `cmd.exe` or from PowerShell.
- Qt, with the Qt Online Installer from <https://www.qt.io/download-qt-installer>.
  Tick there, under the Qt 6 version you want, the entry whose name begins with
  `MinGW`, and under `Build Tools` the MinGW entry of the same version. The first
  is Qt itself, the second the compiler Qt is built with.

Then put the `bin` directory of that Qt on the path of the MSYS2 or Git shell,
the version and the directory being the ones you installed:

```bash
export PATH="/c/Qt/6.11.2/mingw_64/bin:$PATH"
```

**macOS.** The command line tools, and a Qt without a Qt account. Run these in
Terminal, in any directory:

```bash
xcode-select --install
python3 -m pip install --user aqtinstall
python3 -m aqt install-qt mac desktop 6.11.2 clang_64 \
    --outputdir ~/Qt6.11.2 --archives qtbase qttools
export PATH="$HOME/Qt6.11.2/6.11.2/macos/bin:$PATH"
```

Write that last line into `~/.zprofile` to keep it beyond the session. The Qt
Online Installer serves as well, and asks for a Qt account.

**Linux.** The compiler, the Qt of the distribution, and the tool that writes
the run paths. Run the line of your distribution, in any directory:

```bash
sudo apt install build-essential qt6-base-dev patchelf     # Debian, Ubuntu
sudo dnf install gcc-c++ make qt6-qtbase-devel patchelf    # Fedora
sudo pacman -S base-devel qt6-base patchelf                # Arch
```

### Special preconditions for MacOS

- The Qt you install has to be built against an SDK no older than the one your
  Xcode carries. An older Qt asks the linker for frameworks Apple has withdrawn,
  and the link then fails naming a framework that is not found rather than the
  age of the Qt: Qt 6.6.1 fails that way against the SDK of macOS 26, where
  Qt 6.11.2 builds.

- That Qt has to carry the architecture you build for. `deploy.sh` builds for
  the architecture of the machine unless `-a` names another, and refuses the
  deployment where the Qt installation carries something else.

- genc³wb itself asks for the modules `core`, `gui` and `widgets` only, all
  three of which belong to `qtbase`. A Qt installed for this project alone
  therefore needs no more than `qtbase` and the Qt tools, the latter for
  `macdeployqt`, which an installation of the Qt libraries alone does not
  necessarily carry; the other modules of Qt are never built against.

## Getting started

### Installation from source code

Fetch the source and enter it:

```bash
git clone https://github.com/JoergBrueggmann/genc3wb.git
cd genc3wb
```

Build it:

```bash
./deploy.sh
```

`deploy.sh` builds genc³wb, runs the test suite over what it built, and collects
a directory that can be copied to another machine of the same platform and
started there, without Qt or a compiler being installed on it. It builds for the
machine it runs on and does not cross-compile. A build that fails and a test
suite that fails each end the run, so that what is collected is never a failing
build.

The result is left in `dist/genc3wb`. What it holds, and how it is started,
follows the platform:

| Platform | Holds | Started, from within `dist/genc3wb`, with |
| --- | --- | --- |
| Windows | `genc3wb.exe`, the DLLs it imports and `qt.conf` | `./genc3wb.exe` |
| macOS | `genc3wb.app`, the frameworks within it | `./genc3wb.app/Contents/MacOS/genc3wb` |
| Linux | `genc3wb`, `lib/`, `plugins/` and a starter | `./genc3wb.sh` |

On macOS the bundle is started in either of two ways. The commands below are
run in the directory the source was cloned into, that is in `genc3wb`.

Started as a whole, the application is handed to the window server, and what it
writes to standard output does not reach the terminal:

```bash
open dist/genc3wb/genc3wb.app
```

Started as the executable within the bundle, it stays attached to the terminal,
where the increments the code editor writes to standard output can be read:

```bash
dist/genc3wb/genc3wb.app/Contents/MacOS/genc3wb
```

Where macOS refuses a bundle that arrived from another machine, remove the mark
it was given on arrival:

```bash
xattr -dr com.apple.quarantine dist/genc3wb/genc3wb.app
```

`deploy.sh` takes options — among them `-a` for the architecture on macOS,
`-j` for the number of parallel compiler jobs, and `-s` to skip the test suite.
Its own header names them all, and is the authority on them. Read it in the
directory the source was cloned into:

```bash
head -30 deploy.sh
```

### Building without collecting

Where the portable directory is not what is wanted — while developing, say —
qmake and make build the project on their own, into a directory of their own
rather than among the source files, which `.gitignore` keeps out of version
control:

```bash
mkdir -p build/dev
cd build/dev
qmake6 ../../genc3wb.pro CONFIG+=release
make -j4
```

Some installations of Qt name that tool `qmake` rather than `qmake6`; both do
the same here. Raise the `4` of `-j4` to the number of processors you want to
compile with.

That yields the application under `app/` and the test suite under `test/`. The
suite draws no window, and an offscreen platform lets it run without a display.
Run it in `build/dev`, the directory the build was made in:

```bash
QT_QPA_PLATFORM=offscreen ./test/genc3wbtest
```

Neither the frameworks of Qt nor the plugins are carried anywhere by this build:
what it yields runs on this machine, against the Qt that built it, and nowhere
else. `deploy.sh` is what produces something to hand on.



