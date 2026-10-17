#!/usr/bin/env bash
# Builds the workbench and assembles the distribution folder 'dist': under 'bin'
# the executable of the workbench beside the three executables of genc³, so
# that the workbench finds its settings file 'genc3wb.yaml' and the build system
# 'genc3d' in the folder of its own executable; the application bundle
# 'genc3wb.app', whose executable is a symbolic link to 'bin/genc3wb' and which
# declares gc3n-files as the documents it opens, so that a double-click on a
# gc3n-file opens it with the workbench once the bundle is registered; the
# script 'register.command', which registers the bundle with Launch Services
# and puts 'bin' on the search path of the shell; the licence, the description
# and the work items of the package; and, under 'genc3', the rest of the
# distribution of genc³: its description, licence, work items, reference manual
# and examples. What is distributed therefore holds the workbench and the build
# system it drives.
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
sBundleId=de.software4sd.genc3wb

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

if [ ! -d "$sGenc3/dist/bin" ]; then
    echo "FAILED: '$sGenc3/dist/bin' was not assembled" >&2
    exit 1
fi

# the workbench itself
cargo build --release || exit 1

# the version of the package, for the bundle
sVersion="$(grep -m 1 '^version = ' Cargo.toml | sed -E 's/^version = "([^"+]*)(\+([0-9]+))?"/\1.\3/; s/\.$/.0/')"

rm -rf "$sDist"
mkdir -p "$sDist/bin" || exit 1
cp target/release/genc3wb "$sDist/bin/" || exit 1
cp README.md LICENSE CHANGELOG.md "$sDist/" || exit 1

# the distribution of genc³: its executables into 'bin', the rest under 'genc3'
cp -R "$sGenc3/dist" "$sDist/genc3" || exit 1
for sExecutable in genc3d genc3 gc3; do
    mv "$sDist/genc3/bin/$sExecutable" "$sDist/bin/" || exit 1
done
rmdir "$sDist/genc3/bin" || exit 1

# the application bundle: Launch Services starts an application bundle and
# hands the document to it as an event, never a bare executable; the executable
# of the bundle is a symbolic link to the workbench, so that the workbench
# finds its settings file and the build system in 'bin' whichever way it is
# started
sBundle="$sDist/genc3wb.app"
mkdir -p "$sBundle/Contents/MacOS" || exit 1
ln -s ../../../bin/genc3wb "$sBundle/Contents/MacOS/genc3wb" || exit 1
printf 'APPL????' > "$sBundle/Contents/PkgInfo"
cat > "$sBundle/Contents/Info.plist" <<EOT
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleDisplayName</key>
	<string>genc³wb</string>
	<key>CFBundleExecutable</key>
	<string>genc3wb</string>
	<key>CFBundleIdentifier</key>
	<string>$sBundleId</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>genc3wb</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>$sVersion</string>
	<key>CFBundleVersion</key>
	<string>$sVersion</string>
	<key>LSMinimumSystemVersion</key>
	<string>12.0</string>
	<key>NSPrincipalClass</key>
	<string>NSApplication</string>
	<key>NSHighResolutionCapable</key>
	<true/>
	<key>CFBundleDocumentTypes</key>
	<array>
		<dict>
			<key>CFBundleTypeName</key>
			<string>genc³ compiler network</string>
			<key>CFBundleTypeRole</key>
			<string>Editor</string>
			<key>LSHandlerRank</key>
			<string>Owner</string>
			<key>LSItemContentTypes</key>
			<array>
				<string>$sBundleId.gc3n</string>
			</array>
		</dict>
	</array>
	<key>UTExportedTypeDeclarations</key>
	<array>
		<dict>
			<key>UTTypeIdentifier</key>
			<string>$sBundleId.gc3n</string>
			<key>UTTypeDescription</key>
			<string>genc³ compiler network</string>
			<key>UTTypeConformsTo</key>
			<array>
				<string>public.plain-text</string>
			</array>
			<key>UTTypeTagSpecification</key>
			<dict>
				<key>public.filename-extension</key>
				<array>
					<string>gc3n</string>
				</array>
			</dict>
		</dict>
	</array>
</dict>
</plist>
EOT

# the registration: run by the user, by double-click or in a terminal, after
# the distribution was placed where it is to stay, and again after it was moved
cat > "$sDist/register.command" <<'EOT'
#!/usr/bin/env bash
# Registers this distribution of genc³wb on this Mac: the application bundle
# 'genc3wb.app' with Launch Services, so that a double-click on a gc3n-file
# opens it with the workbench, and the folder 'bin' on the search path of the
# shell, so that genc3wb, genc3d, genc3 and gc3 are callable in a terminal.
# Repeatable: run it again after moving the folder.
set -uo pipefail

cd "$(dirname "$0")" || exit 1
sRoot="$(pwd -P)"
sLsregister=/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister
sProfile="$HOME/.zprofile"
sMark='# genc3wb: the folder of the executables of genc³wb and genc³'

if [ ! -x "$sRoot/bin/genc3wb" ] || [ ! -d "$sRoot/genc3wb.app" ]; then
    echo "FAILED: '$sRoot' holds no distribution of genc³wb" >&2
    exit 1
fi

# the bundle: registered by its path; a registration of an earlier location of
# the same bundle identifier is replaced
"$sLsregister" -f "$sRoot/genc3wb.app" || {
    echo "FAILED: the bundle was not registered with Launch Services" >&2
    exit 1
}
echo "registered: $sRoot/genc3wb.app opens gc3n-files"

# the search path: one marked block in the profile of zsh, the shell of macOS,
# replaced where it is present
sBlock="$sMark
export PATH=\"$sRoot/bin:\$PATH\""
touch "$sProfile" || exit 1
if grep -qF "$sMark" "$sProfile"; then
    # the mark and the line after it go, everything else stays
    sKept="$(awk -v mark="$sMark" '$0 == mark { skip = 1; next } skip == 1 { skip = 0; next } { print }' "$sProfile")"
    printf '%s\n\n%s\n' "$sKept" "$sBlock" > "$sProfile" || exit 1
    echo "replaced: the block of genc3wb in $sProfile"
else
    printf '\n%s\n' "$sBlock" >> "$sProfile" || exit 1
    echo "added: the block of genc3wb to $sProfile"
fi
echo "on the search path of new terminals: $sRoot/bin"
EOT
chmod +x "$sDist/register.command" || exit 1

# what was assembled is executable where it was assembled
nStatus=0
for sExecutable in genc3wb genc3d genc3 gc3; do
    if [ ! -x "$sDist/bin/$sExecutable" ]; then
        echo "FAILED: $sDist/bin/$sExecutable is not executable" >&2
        nStatus=1
    fi
done
if [ ! -x "$sBundle/Contents/MacOS/genc3wb" ]; then
    echo "FAILED: the executable of $sBundle does not reach bin/genc3wb" >&2
    nStatus=1
fi
if ! plutil -lint "$sBundle/Contents/Info.plist" > /dev/null 2>&1; then
    echo "FAILED: the Info.plist of $sBundle is malformed" >&2
    nStatus=1
fi
if ! "$sDist/bin/genc3d" --help > /dev/null 2>&1; then
    echo "FAILED: $sDist/bin/genc3d did not run" >&2
    nStatus=1
fi
"$sDist/bin/genc3wb" --help > /dev/null 2>&1
if [ $? -ne 1 ]; then
    echo "FAILED: $sDist/bin/genc3wb did not refuse the option --help with the exit code 1" >&2
    nStatus=1
fi

if [ "$nStatus" -eq 0 ]; then
    echo "OK: $sDist holds the workbench and the build system under bin, the bundle genc3wb.app and register.command"
fi

exit "$nStatus"
