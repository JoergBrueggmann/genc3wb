#!/usr/bin/env bash
#
# deploy.sh — build genc³wb and provide a portable executable.
#
# The result is a directory that can be copied to another machine of the same
# platform and started there, without Qt or a compiler being installed on it.
#
# The script builds for the machine it runs on, and for that machine only: it
# produces Windows code on Windows, Linux code on Linux, and macOS code on
# macOS, on Apple silicon as well as on Intel. It does not cross-compile —
# each platform is built on a machine of that platform.
#
# Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
#
# Usage:
#   ./deploy.sh [-b <build dir>] [-o <output dir>] [-j <jobs>] [-a <arch>] [-s] [-k]
#
#   -b <build dir>    where to build            (default: build/deploy)
#   -o <output dir>   where to put the result   (default: dist/genc3wb)
#   -j <jobs>         parallel compiler jobs    (default: number of processors)
#   -a <arch>         macOS only: arm64, x86_64 or universal
#                                               (default: the architecture of the machine)
#   -s                skip the test suite
#   -k                keep the build directory instead of removing it first
#
# The script fails on the first error, and fails the deployment where the test
# suite fails, so that a failing build is never packaged.

set -euo pipefail

readonly SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
readonly PROJECT_FILE="${SCRIPT_DIR}/genc3wb.pro"

BUILD_DIR="${SCRIPT_DIR}/build/deploy"
OUTPUT_DIR="${SCRIPT_DIR}/dist/genc3wb"
JOBS=""
ARCH_WANTED=""
RUN_TESTS=1
CLEAN_BUILD=1

# ---------------------------------------------------------------- reporting --

say()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
info() { printf '    %s\n' "$*"; }
die()  { printf '\n\033[1;31m!!! %s\033[0m\n' "$*" >&2; exit 1; }

# ------------------------------------------------------------------ options --

while getopts ':b:o:j:a:sk' opt; do
    case "${opt}" in
        b) BUILD_DIR="${OPTARG}" ;;
        o) OUTPUT_DIR="${OPTARG}" ;;
        j) JOBS="${OPTARG}" ;;
        a) ARCH_WANTED="${OPTARG}" ;;
        s) RUN_TESTS=0 ;;
        k) CLEAN_BUILD=0 ;;
        :) die "option -${OPTARG} needs an argument" ;;
        *) die "unknown option -${OPTARG}" ;;
    esac
done

[ -f "${PROJECT_FILE}" ] || die "project file not found: ${PROJECT_FILE}"

# The number of processors is asked of the platform: 'nproc' belongs to the GNU
# tools and is absent on macOS, where the same number is a property of the kernel.
if [ -z "${JOBS}" ]; then
    JOBS="$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)"
fi

# ----------------------------------------------------------------- platform --

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
    Darwin)               PLATFORM="macos"   ;;
    Linux)                PLATFORM="linux"   ;;
    *)                    die "unsupported platform: $(uname -s)" ;;
esac

# The machine names its architecture differently from one platform to the next —
# an Apple silicon Mac says 'arm64' where a Linux machine of the same family says
# 'aarch64'. The names are drawn together here, so that the rest of the script
# speaks of one architecture per family.
case "$(uname -m)" in
    arm64|aarch64)     ARCH="arm64"  ;;
    x86_64|amd64)      ARCH="x86_64" ;;
    *)                 ARCH="$(uname -m)" ;;
esac

if [ -n "${ARCH_WANTED}" ]; then
    [ "${PLATFORM}" = "macos" ] || die "option -a is for macOS only; this machine is ${PLATFORM}"
    case "${ARCH_WANTED}" in
        arm64|x86_64|universal) ;;
        *) die "unknown architecture for -a: ${ARCH_WANTED} (arm64, x86_64 or universal)" ;;
    esac
fi

# The build targets the architecture of the machine unless another one is asked
# for. A Mac of Apple silicon therefore yields arm64 code, and not the x86_64 code
# a Qt installation carrying both slices would otherwise be free to choose.
ARCH_BUILT="${ARCH_WANTED:-${ARCH}}"

# The time a file was written is read differently by the two families of tools:
# GNU date takes the file with -r, where BSD date takes seconds since the epoch
# with the same option and would answer for a path with something else entirely.
# The tool is chosen by the platform rather than by trying one and then the other,
# because each of them answers the call of the other rather than refusing it.
file_time() {
    if [ "${PLATFORM}" = "macos" ]; then
        stat -f '%Sm' -t '%Y-%m-%d %H:%M:%S' -- "$1" 2>/dev/null || echo 'unknown'
    else
        date -r "$1" '+%Y-%m-%d %H:%M:%S' 2>/dev/null || echo 'unknown'
    fi
}

# qmake reports paths in the form of the platform; on Windows that is a path
# with a drive letter, which a POSIX path list would split at its colon. It is
# therefore converted wherever one is put on the path.
to_shell_path() {
    if command -v cygpath >/dev/null 2>&1; then
        cygpath -u -- "$1"
    else
        printf '%s' "$1"
    fi
}

# --------------------------------------------------------------- Qt toolkit --

# QMAKE may be set in the environment to choose a specific Qt installation.
QMAKE="${QMAKE:-}"
if [ -z "${QMAKE}" ]; then
    for candidate in qmake6 qmake; do
        if command -v "${candidate}" >/dev/null 2>&1; then
            QMAKE="$(command -v "${candidate}")"
            break
        fi
    done
fi
[ -n "${QMAKE}" ] || die "qmake not found; install Qt or set QMAKE to its path"

QT_BIN_DIR="$(to_shell_path "$("${QMAKE}" -query QT_INSTALL_BINS)")"
QT_VERSION="$("${QMAKE}" -query QT_VERSION)"

# ------------------------------------------------------------------ toolchain --

# Qt on Windows is built with the toolchain it ships with, against the C runtime
# that toolchain uses. Another compiler of the same name on the path — an MSYS2
# UCRT64 one, for instance — links against a different C runtime, and the link
# then fails on symbols such as '__imp___argc' in libQt6EntryPoint. The toolchain
# is therefore taken from the Qt installation rather than from the path.
TOOLCHAIN_BIN=""

if [ "${PLATFORM}" = "windows" ]; then
    # .../Qt/<version>/<kit>/bin -> .../Qt
    QT_ROOT="$(dirname -- "$(dirname -- "$(dirname -- "${QT_BIN_DIR}")")")"

    for candidate in "${QT_ROOT}"/Tools/mingw*/bin; do
        if [ -x "${candidate}/g++.exe" ] && [ -x "${candidate}/mingw32-make.exe" ]; then
            TOOLCHAIN_BIN="${candidate}"
        fi
    done

    if [ -z "${TOOLCHAIN_BIN}" ]; then
        die "no MinGW toolchain of the Qt installation found under ${QT_ROOT}/Tools
    Qt must be linked with the compiler it ships with. Install the MinGW
    component of Qt, or set MAKE and put its compiler on the path yourself."
    fi

    MAKE="${TOOLCHAIN_BIN}/mingw32-make.exe"
    COMPILER="${TOOLCHAIN_BIN}/g++.exe"
else
    MAKE="${MAKE:-}"
    if [ -z "${MAKE}" ]; then
        for candidate in make gmake; do
            if command -v "${candidate}" >/dev/null 2>&1; then
                MAKE="$(command -v "${candidate}")"
                break
            fi
        done
    fi
    [ -n "${MAKE}" ] || die "no make found; install make"
    TOOLCHAIN_BIN="$(dirname -- "${MAKE}")"

    # Qt on macOS is built with the clang of the command line tools of Xcode, and
    # on Linux with the g++ of the distribution. The compiler is named here rather
    # than assumed, so that the report below tells which one the build used.
    if [ "${PLATFORM}" = "macos" ]; then
        COMPILER="$(command -v clang++ 2>/dev/null || true)"
        [ -n "${COMPILER}" ] || die "no clang++ found; install the command line tools with 'xcode-select --install'"
    else
        COMPILER="$(command -v g++ 2>/dev/null || command -v c++ 2>/dev/null || true)"
        [ -n "${COMPILER}" ] || die "no C++ compiler found; install g++"
    fi
fi

# qmake probes the compiler to learn its include paths, so the toolchain is put
# on the path before it runs, ahead of any other compiler the machine carries.
PATH="${TOOLCHAIN_BIN}:${QT_BIN_DIR}:${PATH}"
export PATH

# --------------------------------------------------------- Qt architecture --

# A Qt installation carries the code of one architecture, or of both where it is
# a universal one. Where it does not carry the architecture being built, the
# build fails late, in the link, on symbols the linker calls undefined without
# saying that the slice is simply missing. The state is therefore named here.
QT_ARCHS=""
if [ "${PLATFORM}" = "macos" ] && command -v lipo >/dev/null 2>&1; then
    QT_LIB_DIR_PROBE="$("${QMAKE}" -query QT_INSTALL_LIBS)"
    for probe in "${QT_LIB_DIR_PROBE}/QtCore.framework/Versions/A/QtCore" \
                 "${QT_LIB_DIR_PROBE}/libQt6Core.dylib" \
                 "${QT_LIB_DIR_PROBE}/libQt5Core.dylib"; do
        if [ -f "${probe}" ]; then
            QT_ARCHS="$(lipo -archs "${probe}" 2>/dev/null || true)"
            break
        fi
    done

    if [ "${ARCH_BUILT}" = "universal" ]; then
        ARCHS_NEEDED="arm64 x86_64"
    else
        ARCHS_NEEDED="${ARCH_BUILT}"
    fi

    if [ -n "${QT_ARCHS}" ]; then
        for wanted in ${ARCHS_NEEDED}; do
            case " ${QT_ARCHS} " in
                *" ${wanted} "*) ;;
                *) die "the Qt installation carries ${QT_ARCHS}, and not ${wanted}
    Install the Qt for ${wanted}, or build for what it carries with -a." ;;
            esac
        done
    fi
fi

say "genc³wb — portable build"
info "platform   : ${PLATFORM}"
info "machine    : ${ARCH}"
info "building   : ${ARCH_BUILT}${QT_ARCHS:+ (Qt carries ${QT_ARCHS})}"
info "qmake      : ${QMAKE} (Qt ${QT_VERSION})"
info "make       : ${MAKE}, ${JOBS} job(s)"
info "compiler   : ${COMPILER}"
info "build dir  : ${BUILD_DIR}"
info "output dir : ${OUTPUT_DIR}"

# -------------------------------------------------------------------- build --

say "Building"

# The build is a release build, and on macOS it is told which architecture to
# produce. Without that, a universal Qt lets the build choose, and the result is
# then not necessarily the code of the machine it is meant to run on.
QMAKE_ARGS=( "CONFIG+=release" )
if [ "${PLATFORM}" = "macos" ]; then
    if [ "${ARCH_BUILT}" = "universal" ]; then
        QMAKE_ARGS+=( "QMAKE_APPLE_DEVICE_ARCHS=arm64 x86_64" )
    else
        QMAKE_ARGS+=( "QMAKE_APPLE_DEVICE_ARCHS=${ARCH_BUILT}" )
    fi
fi

if [ "${CLEAN_BUILD}" -eq 1 ]; then
    rm -rf -- "${BUILD_DIR}"
fi
mkdir -p -- "${BUILD_DIR}"

# The build runs in a subshell; its failure is caught here rather than left to
# a later step, which would otherwise report a missing executable and hide the
# reason it is missing.
if ! (
    cd -- "${BUILD_DIR}"
    "${QMAKE}" "${PROJECT_FILE}" "${QMAKE_ARGS[@]}"
    "${MAKE}" -j"${JOBS}"
); then
    die "the build failed — nothing was packaged
    The last lines above name the reason. A link failing on symbols such as
    '__imp___argc' means the compiler does not match the Qt installation."
fi

# --------------------------------------------------------------------- test --

# The suite is built by the project; a portable build is only packaged where it
# holds, so that a failing build is never shipped.
if [ "${RUN_TESTS}" -eq 1 ]; then
    say "Running the test suite"

    TEST_EXE="$(find "${BUILD_DIR}/test" -maxdepth 2 -type f -name 'genc3wbtest*' | head -n 1)"
    [ -n "${TEST_EXE}" ] || die "test executable not found under ${BUILD_DIR}/test"

    # The suite draws no window; an offscreen platform lets it run without a display.
    if QT_QPA_PLATFORM=offscreen "${TEST_EXE}"; then
        info "the test suite held"
    else
        die "the test suite failed — nothing was packaged"
    fi
else
    say "Skipping the test suite (-s)"
fi

# ------------------------------------------------------------------ collect --

say "Collecting the portable executable"

# What the build yields is not the same thing on every platform: Windows and
# Linux yield one executable file, where macOS yields an application bundle — a
# directory carrying the executable, its resources and, after macdeployqt, the
# frameworks of Qt. The bundle is what is copied and what the user starts, so it
# is taken as a whole; the executable inside it is named as well, because the
# tools that read what a binary imports read that file and not the directory.
if [ "${PLATFORM}" = "macos" ]; then
    APP_ARTEFACT="$(find "${BUILD_DIR}/app" -maxdepth 2 -type d -name 'genc3wb.app' | head -n 1)"
    [ -n "${APP_ARTEFACT}" ] || die "application bundle not found under ${BUILD_DIR}/app"

    APP_NAME="$(basename -- "${APP_ARTEFACT}")"
    APP_BINARY_REL="Contents/MacOS/genc3wb"
else
    APP_ARTEFACT="$(find "${BUILD_DIR}/app" -maxdepth 2 -type f \( -name 'genc3wb' -o -name 'genc3wb.exe' \) | head -n 1)"
    [ -n "${APP_ARTEFACT}" ] || die "application not found under ${BUILD_DIR}/app"

    APP_NAME="$(basename -- "${APP_ARTEFACT}")"
    APP_BINARY_REL=""
fi

# A running application holds its own file and the directory it lies in, so the
# removal below would fail and end the deployment with the message of the remove
# command, leaving the bundle of an earlier run in place. That earlier bundle
# then looks like the result of this run. The state is therefore named here.
if [ -e "${OUTPUT_DIR}" ]; then
    if ! rm -rf -- "${OUTPUT_DIR}" 2>/dev/null; then
        die "the output directory cannot be replaced: ${OUTPUT_DIR}
    Close ${APP_NAME} where it is still running, and close any window of the
    file manager showing that directory, then run this script again.
    The bundle of the earlier run is left untouched."
    fi
fi
mkdir -p -- "${OUTPUT_DIR}"
cp -R -- "${APP_ARTEFACT}" "${OUTPUT_DIR}/"

# The path of what is copied, and the path of the binary within it. On Windows
# and Linux the two are the same file; on macOS the first is the bundle.
APP_PATH="${OUTPUT_DIR}/${APP_NAME}"
APP_BINARY="${APP_PATH}${APP_BINARY_REL:+/${APP_BINARY_REL}}"

info "application: ${APP_NAME}"

case "${PLATFORM}" in

windows)
    WINDEPLOYQT="${QT_BIN_DIR}/windeployqt6"
    [ -x "${WINDEPLOYQT}" ] || WINDEPLOYQT="${QT_BIN_DIR}/windeployqt"
    [ -x "${WINDEPLOYQT}" ] || die "windeployqt not found in ${QT_BIN_DIR}"

    # --compiler-runtime brings the MinGW or MSVC runtime libraries along, which
    # the target machine does not necessarily carry.
    #
    # Neither the software renderer nor the D3D compiler is excluded. They cost
    # some twenty megabytes, and without them the platform plugin is found but
    # cannot be initialised on a machine whose graphics driver offers no usable
    # OpenGL: it then reports that no Qt platform plugin could be initialized.
    "${WINDEPLOYQT}" \
        --release \
        --compiler-runtime \
        --no-translations \
        "${APP_PATH}"

    # The plugins are found beside the application and nowhere else, so that a
    # QT_PLUGIN_PATH, or a Qt installation on the target machine, cannot divert
    # the application to the plugins of another build.
    {
        echo "[Paths]"
        echo "Prefix = ."
        echo "Plugins = ."
        echo "Libraries = ."
        echo "Imports = ."
    } > "${OUTPUT_DIR}/qt.conf"
    info "wrote qt.conf, pinning the plugins to the output directory"

    # windeployqt resolves what the executable itself imports. The MinGW runtime
    # libraries are loaded by the Qt libraries rather than by the application, so
    # they are copied here wherever windeployqt did not bring them.
    for lib in libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll; do
        if [ ! -f "${OUTPUT_DIR}/${lib}" ] && [ -f "${TOOLCHAIN_BIN}/${lib}" ]; then
            cp -- "${TOOLCHAIN_BIN}/${lib}" "${OUTPUT_DIR}/"
            info "added ${lib}"
        fi
    done
    ;;

macos)
    MACDEPLOYQT="${QT_BIN_DIR}/macdeployqt"
    [ -x "${MACDEPLOYQT}" ] || die "macdeployqt not found in ${QT_BIN_DIR}"

    # macdeployqt copies the frameworks and the plugins into the bundle and writes
    # the load paths of the executable to point at them.
    "${MACDEPLOYQT}" "${APP_PATH}" -always-overwrite

    # A binary of Apple silicon is loaded only where its signature holds, and
    # macdeployqt breaks the signature of everything whose load paths it wrote.
    # The bundle is therefore signed again, ad hoc: that carries no identity and
    # no notarisation — the target machine still asks about an application from
    # an unidentified developer — but the code loads, which without a signature
    # it would not do at all.
    if command -v codesign >/dev/null 2>&1; then
        if codesign --force --deep --sign - --timestamp=none "${APP_PATH}" 2>/dev/null; then
            info "signed the bundle ad hoc, so that it loads on Apple silicon"
        else
            info "the bundle could not be signed ad hoc; on Apple silicon it may refuse to start"
        fi
    fi
    ;;

linux)
    # Qt ships no deployment tool for Linux. The libraries the executable needs
    # are copied beside it, and a starter sets the library path to that directory.
    QT_LIB_DIR="$("${QMAKE}" -query QT_INSTALL_LIBS)"
    QT_PLUGIN_DIR="$("${QMAKE}" -query QT_INSTALL_PLUGINS)"

    mkdir -p -- "${OUTPUT_DIR}/lib" "${OUTPUT_DIR}/plugins"

    # The libraries of the operating system are left to the target machine, which
    # carries its own — they are bound to its graphics driver and to its kernel,
    # and a copy brought from elsewhere would be the wrong one. These few are the
    # exception: they are thin wrappers over the protocol of the window system,
    # bound to nothing of the machine, and the plugin of X11 asks for them while
    # a machine that was installed without a development environment does not
    # carry them. Without them the plugin is found but cannot be loaded, and the
    # application ends saying that no Qt platform plugin could be initialized —
    # naming xcb, and not the library that is actually absent.
    XCB_HELPERS="libxcb-cursor.so.0 \
                 libxcb-icccm.so.4 \
                 libxcb-image.so.0 \
                 libxcb-keysyms.so.1 \
                 libxcb-render-util.so.0 \
                 libxcb-util.so.1 \
                 libxkbcommon-x11.so.0"

    # The platform plugins are named one by one rather than taken as a directory.
    # The directory carries plugins for targets this application is not built for
    # — a VNC server, a frame buffer of an embedded device — and each of them
    # imports libraries of its own: the VNC one imports the network library of Qt,
    # which imports Kerberos, which a desktop machine need not carry at all. The
    # deployment would then fail on a library that no plugin of this application
    # ever asks for. What is named here is the window systems of a desktop, and
    # the two plugins that draw no window, which the test suite uses.
    #
    # X11, Wayland, and the two that draw no window.
    mkdir -p -- "${OUTPUT_DIR}/plugins/platforms"
    PLATFORM_PLUGIN_COUNT=0
    for plugin in libqxcb libqwayland-generic libqwayland-egl libqoffscreen libqminimal; do
        if [ -f "${QT_PLUGIN_DIR}/platforms/${plugin}.so" ]; then
            cp -- "${QT_PLUGIN_DIR}/platforms/${plugin}.so" "${OUTPUT_DIR}/plugins/platforms/"
            PLATFORM_PLUGIN_COUNT=$((PLATFORM_PLUGIN_COUNT + 1))
        fi
    done
    [ "${PLATFORM_PLUGIN_COUNT}" -gt 0 ] || die "no platform plugin found under ${QT_PLUGIN_DIR}/platforms"
    info "carried ${PLATFORM_PLUGIN_COUNT} platform plugin(s)"

    # The groups the platform plugins and the widgets draw upon: the image formats,
    # without which the indicator of the processing state draws nothing; the icon
    # engines; what X11 and Wayland need beside their platform plugin; and the
    # input contexts, without which a keyboard of another script cannot be used.
    # What the installation does not carry is passed over rather than failing.
    for group in imageformats iconengines xcbglintegrations platforminputcontexts \
                 wayland-decoration-client wayland-graphics-integration-client \
                 wayland-shell-integration; do
        if [ -d "${QT_PLUGIN_DIR}/${group}" ]; then
            cp -R -- "${QT_PLUGIN_DIR}/${group}" "${OUTPUT_DIR}/plugins/"
        fi
    done

    # A plugin imports libraries of its own that the executable does not import —
    # the platform plugin imports the Qt library of the window system, which in
    # turn imports others. Copying only what the executable imports therefore
    # yields a bundle that starts nowhere but on the machine that built it. The
    # binaries of the output directory are read again and again, until a pass
    # copies nothing further, so that the closure is complete.
    #
    # A plugin carries the run path of the installation it was built in, which is
    # relative to the plugin itself: from the copy in the output directory it now
    # names the library directory of the bundle, which is filling up as this runs.
    # A library not yet copied is therefore reported as not found rather than
    # resolved into the installation, and would be passed over silently. The
    # library directory of the installation is named to the loader for the reading,
    # so that what is missing resolves there, and each library is looked for by its
    # name as well, so that a plugin whose run path names nothing at all is served.
    while :; do
        COPIED=0
        while IFS= read -r binary; do
            [ -n "${binary}" ] || continue
            while IFS=' ' read -r name path; do
                [ -n "${name}" ] || continue

                # What the loader answered, or the library of that name in the
                # installation where it answered nothing.
                case "${path}" in
                    "${QT_LIB_DIR}"/*) lib="${path}" ;;
                    *) lib="${QT_LIB_DIR}/${name}" ;;
                esac

                # A helper of the window system lies where the machine keeps its
                # own libraries, and not in the installation of Qt; it is taken
                # from where the loader found it.
                if [ ! -f "${lib}" ]; then
                    case " ${XCB_HELPERS} " in
                        *" ${name} "*)
                            if [ -f "${path}" ]; then
                                lib="${path}"
                            fi
                            ;;
                    esac
                fi
                [ -f "${lib}" ] || continue

                if [ ! -f "${OUTPUT_DIR}/lib/${name}" ]; then
                    cp -L -- "${lib}" "${OUTPUT_DIR}/lib/${name}"
                    info "added ${name}"
                    COPIED=1
                fi
            done < <(LD_LIBRARY_PATH="${QT_LIB_DIR}" ldd "${binary}" 2>/dev/null \
                         | awk '/=>/ {
                                    if ($0 ~ /not found/)  { print $1, "not found" }
                                    else if ($3 ~ /^\//)   { print $1, $3 }
                                }')
        done < <(find "${APP_BINARY}" "${OUTPUT_DIR}/lib" "${OUTPUT_DIR}/plugins" -type f \
                     \( -name '*.so' -o -name '*.so.*' -o -perm -u+x \) 2>/dev/null)

        [ "${COPIED}" -eq 1 ] || break
    done

    # What the executable and the libraries carry as their run path still names the
    # Qt installation of this machine, and that run path decides where the loader
    # looks. Which of the two tags the linker wrote decides whether it can be
    # overruled: the older tag, RPATH, is searched before the library path of the
    # environment, so the starter cannot direct the application to the libraries
    # carried here; the newer tag, RUNPATH, is searched after it, so it can. Both
    # are produced by the same sources and the same flags, and the deployment must
    # not rest on which one a machine happens to write.
    #
    # The run path is therefore written anew, relative to the file that carries it,
    # so that each binary names the library directory of the bundle and nothing of
    # this machine. '$ORIGIN' is resolved by the loader to the directory of the
    # binary being loaded, and is kept from the shell by the quotes.
    RUNPATH_TOOL=""
    for candidate in patchelf chrpath; do
        if command -v "${candidate}" >/dev/null 2>&1; then
            RUNPATH_TOOL="${candidate}"
            break
        fi
    done

    if [ -n "${RUNPATH_TOOL}" ]; then
        RUNPATH_COUNT=0
        while IFS= read -r binary; do
            [ -n "${binary}" ] || continue

            # Where the binary lies decides how it reaches the library directory.
            case "${binary}" in
                "${OUTPUT_DIR}/lib/"*)     origin='$ORIGIN' ;;
                "${OUTPUT_DIR}/plugins/"*) origin='$ORIGIN/../../lib' ;;
                *)                         origin='$ORIGIN/lib' ;;
            esac

            # Only what still names this machine is written anew; the libraries of
            # Qt already name their own directory and are left as they are.
            if ! readelf -d "${binary}" 2>/dev/null | grep -qF "${QT_LIB_DIR}"; then
                continue
            fi

            if [ "${RUNPATH_TOOL}" = "patchelf" ]; then
                patchelf --set-rpath "${origin}" "${binary}" 2>/dev/null || continue
            else
                # chrpath writes into the room the old run path took, which the
                # path of an installation always affords for a relative one.
                chrpath -r "${origin}" "${binary}" >/dev/null 2>&1 || continue
            fi
            RUNPATH_COUNT=$((RUNPATH_COUNT + 1))
        done < <(find "${APP_BINARY}" "${OUTPUT_DIR}/lib" "${OUTPUT_DIR}/plugins" -type f \
                     \( -name '*.so' -o -name '*.so.*' -o -perm -u+x \) 2>/dev/null)

        info "wrote the run path of ${RUNPATH_COUNT} binary/binaries, with ${RUNPATH_TOOL}"
    else
        info "neither patchelf nor chrpath is installed; the run path still names this machine"
        info "  the starter directs the application to the libraries carried along, which"
        info "  holds wherever the target machine carries no Qt of its own"
    fi

    # The starter names the directories of the bundle rather than leaving them to
    # be found: a Qt installed on the target machine would otherwise be taken in
    # place of the one carried here, and the two need not be of the same version.
    {
        echo "#!/usr/bin/env bash"
        echo 'here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"'
        echo 'export LD_LIBRARY_PATH="${here}/lib:${LD_LIBRARY_PATH:-}"'
        echo 'export QT_PLUGIN_PATH="${here}/plugins"'
        echo 'exec "${here}/genc3wb" "$@"'
    } > "${OUTPUT_DIR}/genc3wb.sh"
    chmod +x -- "${OUTPUT_DIR}/genc3wb.sh"
    info "wrote genc3wb.sh, which names the libraries and the plugins of the bundle"
    ;;

esac

# ------------------------------------------------------------------- verify --

say "Verifying that the result carries what it needs"

# The application is not started: it opens a window and runs an event loop, so
# starting it would say nothing about portability and would not end by itself.
# Instead every binary of the output directory is read, and each library it
# imports must either lie in the output directory or belong to the operating
# system. What is left over is what the target machine would miss. Each platform
# names its own tool for that reading: objdump on Windows, ldd on Linux, otool
# on macOS.
if [ "${PLATFORM}" = "windows" ] && command -v objdump >/dev/null 2>&1; then

    SYSTEM_DIR="$(to_shell_path "${SYSTEMROOT:-C:/Windows}")/System32"
    MISSING=""
    IMPORT_COUNT=0

    while IFS= read -r binary; do
        [ -n "${binary}" ] || continue
        while IFS= read -r imported; do
            [ -n "${imported}" ] || continue
            IMPORT_COUNT=$((IMPORT_COUNT + 1))
            if find "${OUTPUT_DIR}" -type f -iname "${imported}" | grep -q . ; then
                continue
            fi
            if [ -f "${SYSTEM_DIR}/${imported}" ]; then
                continue
            fi
            case "${imported}" in
                api-ms-win-*|ext-ms-*) continue ;;
            esac
            case " ${MISSING} " in
                *" ${imported} "*) ;;
                *) MISSING="${MISSING} ${imported}" ;;
            esac
        done < <(objdump -p "${binary}" 2>/dev/null | awk '/DLL Name:/ { print $3 }')
    done < <(find "${OUTPUT_DIR}" -type f \( -iname '*.exe' -o -iname '*.dll' \))

    info "imports checked: ${IMPORT_COUNT}"
    if [ -n "${MISSING}" ]; then
        die "the target machine would miss:${MISSING}"
    fi
    info "every imported library is carried along or belongs to the operating system"

    # The platform plugin is what fails first where a deployment is incomplete.
    [ -f "${OUTPUT_DIR}/platforms/qwindows.dll" ] || die "platforms/qwindows.dll is missing"
    [ -f "${OUTPUT_DIR}/qt.conf" ] || die "qt.conf is missing"
    info "platform plugin and qt.conf are in place"

elif [ "${PLATFORM}" = "linux" ] && command -v ldd >/dev/null 2>&1; then

    # The bundle is read as the starter would present it: with its own library
    # directory named first. A library reported as not found is one the target
    # machine would have to carry itself, and one still answered from the Qt
    # installation is one that was not copied — on a machine without Qt it would
    # be missing. Both are named.
    MISSING=""
    OUTSIDE=""
    PREFERRED=""
    NOT_CARRIED=""
    IMPORT_COUNT=0
    QT_LIB_DIR="$("${QMAKE}" -query QT_INSTALL_LIBS)"

    while IFS= read -r binary; do
        [ -n "${binary}" ] || continue
        while IFS= read -r line; do
            [ -n "${line}" ] || continue
            IMPORT_COUNT=$((IMPORT_COUNT + 1))
            name="${line%% *}"
            path="${line#* }"
            case "${path}" in
                "not found")
                    case " ${MISSING} " in
                        *" ${name} "*) ;;
                        *) MISSING="${MISSING} ${name}" ;;
                    esac
                    ;;
                "${QT_LIB_DIR}"/*)
                    # Answered from the installation. Where no library of that
                    # name was carried along, the target machine would miss it,
                    # and that ends the deployment. Where one was carried, only
                    # the order of the search on this machine put the installation
                    # first — a machine that carries no Qt has nothing to put
                    # first, and takes the one carried along.
                    if [ -f "${OUTPUT_DIR}/lib/${name}" ]; then
                        case " ${PREFERRED} " in
                            *" ${name} "*) ;;
                            *) PREFERRED="${PREFERRED} ${name}" ;;
                        esac
                    else
                        case " ${OUTSIDE} " in
                            *" ${name} "*) ;;
                            *) OUTSIDE="${OUTSIDE} ${name}" ;;
                        esac
                    fi
                    ;;
            esac

            # A helper of the window system that is still answered from the
            # machine is one that was not carried along. It is present here, and
            # the deployment would hold; on a machine that does not carry it the
            # application would end before it draws its window.
            case " ${XCB_HELPERS:-} " in
                *" ${name} "*)
                    case "${path}" in
                        "${OUTPUT_DIR}"/*|"not found") ;;
                        *)
                            case " ${NOT_CARRIED} " in
                                *" ${name} "*) ;;
                                *) NOT_CARRIED="${NOT_CARRIED} ${name}" ;;
                            esac
                            ;;
                    esac
                    ;;
            esac
        done < <(LD_LIBRARY_PATH="${OUTPUT_DIR}/lib" ldd "${binary}" 2>/dev/null \
                     | awk '/=>/ {
                                if ($0 ~ /not found/)  { print $1, "not found" }
                                else if ($3 ~ /^\//)   { print $1, $3 }
                            }')
    done < <(find "${APP_BINARY}" "${OUTPUT_DIR}/lib" "${OUTPUT_DIR}/plugins" -type f \
                 \( -name '*.so' -o -name '*.so.*' -o -perm -u+x \) 2>/dev/null)

    info "imports checked: ${IMPORT_COUNT}"
    [ -z "${MISSING}" ] || die "the target machine would miss:${MISSING}"
    [ -z "${OUTSIDE}" ] || die "not carried along, and answered here only by the Qt installation:${OUTSIDE}"
    [ -z "${NOT_CARRIED}" ] || die "helpers of the window system still answered from this machine:${NOT_CARRIED}"
    info "every imported library is carried along or belongs to the operating system"

    # Carried along, and still answered from the installation on this machine.
    # The bundle holds on a machine without Qt, where nothing else answers, but
    # what is read here is then not what the target machine would load.
    if [ -n "${PREFERRED}" ]; then
        info "answered here from the Qt installation, though carried along:${PREFERRED}"
        info "  the run path of this machine names the installation, and could not be"
        info "  written anew; install patchelf or chrpath to have the bundle name itself"
    fi

    # The platform plugin is what fails first where a deployment is incomplete.
    [ -f "${OUTPUT_DIR}/plugins/platforms/libqxcb.so" ] \
        || [ -f "${OUTPUT_DIR}/plugins/platforms/libqwayland-generic.so" ] \
        || die "no platform plugin under plugins/platforms"
    [ -x "${OUTPUT_DIR}/genc3wb.sh" ] || die "the starter genc3wb.sh is missing"
    info "platform plugin and starter are in place"

elif [ "${PLATFORM}" = "macos" ] && command -v otool >/dev/null 2>&1; then

    # A load path pointing into the Qt installation is one macdeployqt did not
    # rewrite; on a machine without Qt the bundle would then not load at all.
    OUTSIDE=""
    IMPORT_COUNT=0
    QT_LIB_DIR="$("${QMAKE}" -query QT_INSTALL_LIBS)"
    QT_PREFIX="$("${QMAKE}" -query QT_INSTALL_PREFIX)"

    while IFS= read -r binary; do
        [ -n "${binary}" ] || continue
        while IFS= read -r loaded; do
            [ -n "${loaded}" ] || continue
            IMPORT_COUNT=$((IMPORT_COUNT + 1))
            case "${loaded}" in
                "${QT_LIB_DIR}"/*|"${QT_PREFIX}"/*)
                    case " ${OUTSIDE} " in
                        *" ${loaded} "*) ;;
                        *) OUTSIDE="${OUTSIDE} ${loaded}" ;;
                    esac
                    ;;
            esac
        done < <(otool -L "${binary}" 2>/dev/null | awk 'NR > 1 { print $1 }')
    done < <(find "${APP_PATH}" -type f \( -perm -u+x -o -name '*.dylib' \) 2>/dev/null)

    info "load paths checked: ${IMPORT_COUNT}"
    [ -z "${OUTSIDE}" ] || die "still loaded from the Qt installation, and not carried along:${OUTSIDE}"
    info "every framework and library is carried along or belongs to the operating system"

    # The platform plugin is what fails first where a deployment is incomplete.
    [ -f "${APP_PATH}/Contents/PlugIns/platforms/libqcocoa.dylib" ] \
        || die "Contents/PlugIns/platforms/libqcocoa.dylib is missing"

    # The architecture is asserted rather than assumed: a bundle of x86_64 runs on
    # Apple silicon only through Rosetta, and not at all where that is absent.
    if command -v lipo >/dev/null 2>&1; then
        APP_ARCHS="$(lipo -archs "${APP_BINARY}" 2>/dev/null || echo 'unknown')"
        info "architecture of the executable: ${APP_ARCHS}"
        if [ "${ARCH_BUILT}" != "universal" ]; then
            case " ${APP_ARCHS} " in
                *" ${ARCH_BUILT} "*) ;;
                *) die "the executable carries ${APP_ARCHS}, and not the ${ARCH_BUILT} that was asked for" ;;
            esac
        fi
    fi

    # An invalid signature keeps the bundle from loading on Apple silicon, which
    # is worth knowing here rather than on the target machine.
    if command -v codesign >/dev/null 2>&1; then
        if codesign --verify --deep "${APP_PATH}" 2>/dev/null; then
            info "the signature of the bundle holds"
        else
            info "the signature of the bundle does not hold; on Apple silicon it may refuse to start"
        fi
    fi

else
    info "no import check on this platform; verify the result on a machine without Qt"
fi

FILE_COUNT="$(find "${OUTPUT_DIR}" -type f | wc -l)"
TOTAL_SIZE="$(du -sh "${OUTPUT_DIR}" | cut -f1)"
APP_SIZE="$(du -sh -- "${APP_PATH}" | cut -f1)"
APP_TIME="$(file_time "${APP_BINARY}")"

say "Done"
info "directory : ${OUTPUT_DIR}"
info "files     : ${FILE_COUNT}"
info "size      : ${TOTAL_SIZE}"
info ""
# The executable is named with its size and its time, so that a bundle of an
# earlier run is recognised as such rather than started as the result of this one.
info "executable: ${APP_NAME}"
info "  built   : ${PLATFORM}, ${ARCH_BUILT}"
info "  size    : ${APP_SIZE}"
info "  written : ${APP_TIME}"
info ""
info "Copy the whole directory to a ${PLATFORM} machine of ${ARCH_BUILT} and start it there:"

case "${PLATFORM}" in
    windows) info "  start ${APP_NAME} in the directory." ;;
    linux)   info "  start genc3wb.sh, which names the libraries and the plugins carried along." ;;
    macos)   info "  start ${APP_NAME}. Where the machine refuses a bundle that came from another"
             info "  machine, remove the mark it was given on arrival:"
             info "    xattr -dr com.apple.quarantine ${APP_NAME}" ;;
esac
