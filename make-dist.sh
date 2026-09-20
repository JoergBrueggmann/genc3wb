#!/usr/bin/env bash
# Builds the workbench and assembles the distribution folder 'dist': the
# executable of the workbench, the licence, the description and the work items
# of the package, and beside them, under 'genc3', a copy of the distribution of
# genc³ with its build system 'genc3d'. What is distributed therefore holds the
# workbench and the build system it drives.
#
# The distribution of genc³ is made first, by its own script 'make-dist.sh',
# so that the copy is the one that was just built and verified there. The
# project folder of genc³ is not named here, since this script belongs to the
# code repository: it is given as the first argument, or in the environment
# variable GENC3.
set -uo pipefail

cd "$(dirname "$0")" || exit 1

sDist=./dist
sGenc3="${1:-${GENC3:-}}"

if [ -z "$sGenc3" ]; then
    echo "usage: $0 [<project folder of genc3>]" >&2
    echo "       the folder is taken from GENC3 where no argument is given" >&2
    exit 1
fi

if [ ! -x "$sGenc3/make-dist.sh" ]; then
    echo "FAILED: '$sGenc3' holds no executable 'make-dist.sh'" >&2
    exit 1
fi

# the distribution of genc³, made by the script of that project
"$sGenc3/make-dist.sh" || { echo "FAILED: the distribution of genc3 was not made" >&2; exit 1; }

if [ ! -d "$sGenc3/dist" ]; then
    echo "FAILED: '$sGenc3/dist' was not assembled" >&2
    exit 1
fi

# the workbench itself
cargo build --release || exit 1

rm -rf "$sDist"
mkdir -p "$sDist/bin" || exit 1
cp target/release/genc3wb "$sDist/bin/" || exit 1
cp README.md LICENSE CHANGELOG.md "$sDist/" || exit 1
cp -R "$sGenc3/dist" "$sDist/genc3" || exit 1

# what was assembled is executable where it was assembled
nStatus=0
if [ ! -x "$sDist/bin/genc3wb" ]; then
    echo "FAILED: $sDist/bin/genc3wb is not executable" >&2
    nStatus=1
fi
if ! "$sDist/genc3/bin/genc3d" --help > /dev/null 2>&1; then
    echo "FAILED: $sDist/genc3/bin/genc3d did not run" >&2
    nStatus=1
fi

if [ "$nStatus" -eq 0 ]; then
    echo "OK: $sDist holds the workbench and, under genc3, the build system"
fi

exit "$nStatus"
