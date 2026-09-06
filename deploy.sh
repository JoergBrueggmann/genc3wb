#!/usr/bin/env bash
#
# deploy.sh — build genc³wb and provide a portable executable.
#
# The result is a directory that can be copied to another machine of the same
# platform and started there, without Qt or a compiler being installed on it.
#
# Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
#
# Usage:
#   ./deploy.sh [-b <build dir>] [-o <output dir>] [-j <jobs>] [-s] [-k]
#
#   -b <build dir>    where to build            (default: build/deploy)
#   -o <output dir>   where to put the result   (default: dist/genc3wb)
#   -j <jobs>         parallel compiler jobs    (default: number of processors)
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
RUN_TESTS=1
CLEAN_BUILD=1

# ---------------------------------------------------------------- reporting --

say()  { printf '\n\033[1m==> %s\033[0m\n' "$*"; }
info() { printf '    %s\n' "$*"; }
die()  { printf '\n\033[1;31m!!! %s\033[0m\n' "$*" >&2; exit 1; }

# ------------------------------------------------------------------ options --

while getopts ':b:o:j:sk' opt; do
    case "${opt}" in
        b) BUILD_DIR="${OPTARG}" ;;
        o) OUTPUT_DIR="${OPTARG}" ;;
        j) JOBS="${OPTARG}" ;;
        s) RUN_TESTS=0 ;;
        k) CLEAN_BUILD=0 ;;
        :) die "option -${OPTARG} needs an argument" ;;
        *) die "unknown option -${OPTARG}" ;;
    esac
done

[ -f "${PROJECT_FILE}" ] || die "project file not found: ${PROJECT_FILE}"

if [ -z "${JOBS}" ]; then
    JOBS="$(nproc 2>/dev/null || echo 4)"
fi

# ----------------------------------------------------------------- platform --

case "$(uname -s)" in
    MINGW*|MSYS*|CYGWIN*) PLATFORM="windows" ;;
    Darwin)               PLATFORM="macos"   ;;
    Linux)                PLATFORM="linux"   ;;
    *)                    die "unsupported platform: $(uname -s)" ;;
esac

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
fi

# qmake probes the compiler to learn its include paths, so the toolchain is put
# on the path before it runs, ahead of any other compiler the machine carries.
PATH="${TOOLCHAIN_BIN}:${QT_BIN_DIR}:${PATH}"
export PATH

say "genc³wb — portable build"
info "platform   : ${PLATFORM}"
info "qmake      : ${QMAKE} (Qt ${QT_VERSION})"
info "make       : ${MAKE}, ${JOBS} job(s)"
info "compiler   : $(command -v g++ 2>/dev/null || echo 'not found')"
info "build dir  : ${BUILD_DIR}"
info "output dir : ${OUTPUT_DIR}"

# -------------------------------------------------------------------- build --

say "Building"

if [ "${CLEAN_BUILD}" -eq 1 ]; then
    rm -rf -- "${BUILD_DIR}"
fi
mkdir -p -- "${BUILD_DIR}"

# The build runs in a subshell; its failure is caught here rather than left to
# a later step, which would otherwise report a missing executable and hide the
# reason it is missing.
if ! (
    cd -- "${BUILD_DIR}"
    "${QMAKE}" "${PROJECT_FILE}" CONFIG+=release
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

APP_EXE="$(find "${BUILD_DIR}/app" -maxdepth 2 -type f \( -name 'genc3wb' -o -name 'genc3wb.exe' \) | head -n 1)"
[ -n "${APP_EXE}" ] || die "application not found under ${BUILD_DIR}/app"

APP_NAME="$(basename -- "${APP_EXE}")"

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
cp -- "${APP_EXE}" "${OUTPUT_DIR}/"

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
        "${OUTPUT_DIR}/${APP_NAME}"

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
    "${MACDEPLOYQT}" "${OUTPUT_DIR}/${APP_NAME}.app" -always-overwrite
    ;;

linux)
    # Qt ships no deployment tool for Linux. The libraries the executable needs
    # are copied beside it, and a starter sets the library path to that directory.
    QT_LIB_DIR="$("${QMAKE}" -query QT_INSTALL_LIBS)"
    QT_PLUGIN_DIR="$("${QMAKE}" -query QT_INSTALL_PLUGINS)"

    mkdir -p -- "${OUTPUT_DIR}/lib" "${OUTPUT_DIR}/plugins"
    ldd "${OUTPUT_DIR}/${APP_NAME}" \
        | awk '{ print $3 }' \
        | grep -F "${QT_LIB_DIR}" \
        | while read -r lib; do cp -Lv -- "${lib}" "${OUTPUT_DIR}/lib/"; done
    cp -R -- "${QT_PLUGIN_DIR}/platforms" "${OUTPUT_DIR}/plugins/"

    {
        echo "#!/usr/bin/env bash"
        echo 'here="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"'
        echo 'export LD_LIBRARY_PATH="${here}/lib:${LD_LIBRARY_PATH:-}"'
        echo 'export QT_PLUGIN_PATH="${here}/plugins"'
        echo 'exec "${here}/genc3wb" "$@"'
    } > "${OUTPUT_DIR}/genc3wb.sh"
    chmod +x -- "${OUTPUT_DIR}/genc3wb.sh"
    ;;

esac

# ------------------------------------------------------------------- verify --

say "Verifying that the result carries what it needs"

# The application is not started: it opens a window and runs an event loop, so
# starting it would say nothing about portability and would not end by itself.
# Instead every binary of the output directory is read, and each library it
# imports must either lie in the output directory or belong to the operating
# system. What is left over is what the target machine would miss.
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
else
    info "no import check on this platform; verify the result on a machine without Qt"
fi

FILE_COUNT="$(find "${OUTPUT_DIR}" -type f | wc -l)"
TOTAL_SIZE="$(du -sh "${OUTPUT_DIR}" | cut -f1)"
APP_SIZE="$(du -h -- "${OUTPUT_DIR}/${APP_NAME}" | cut -f1)"
APP_TIME="$(date -r "${OUTPUT_DIR}/${APP_NAME}" '+%Y-%m-%d %H:%M:%S')"

say "Done"
info "directory : ${OUTPUT_DIR}"
info "files     : ${FILE_COUNT}"
info "size      : ${TOTAL_SIZE}"
info ""
# The executable is named with its size and its time, so that a bundle of an
# earlier run is recognised as such rather than started as the result of this one.
info "executable: ${APP_NAME}"
info "  size    : ${APP_SIZE}"
info "  written : ${APP_TIME}"
info ""
info "Copy the whole directory to the target machine and start ${APP_NAME} in it."
