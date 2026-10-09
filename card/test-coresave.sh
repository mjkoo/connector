#!/bin/sh
set -eu

HERE="$(cd "$(dirname "$0")" && pwd)"
SCRIPT="$HERE/System/coresave.sh"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

setup() {
	rm -rf "$TMP/card" "$TMP/mountpoint"
	mkdir -p "$TMP/card"
	cat > "$TMP/mountpoint" <<-SH
		#!/bin/sh
		exit ${1:-0}
	SH
	chmod +x "$TMP/mountpoint"
}

run() {
	printf '%s' "$2" | AGS_CARD="$TMP/card" AGS_MOUNTPOINT="$TMP/mountpoint" \
		AGS_CORE_EXE="$1" AGS_CORE_CMDLINE="${CMDLINE:-$1}" sh "$SCRIPT" "${3:-4321}" "${4:-11}" "${5:-slot-emu}" 0
}

setup
run /usr/bin/busybox "not a frontend core"
grep -q 'signal=11 thread=slot-emu exe=/usr/bin/busybox' "$TMP/card/crash/cores.log" \
	|| { echo "no log line for a non-frontend crash: [$(cat "$TMP/card/crash/cores.log")]" >&2; exit 1; }
[ ! -f "$TMP/card/crash/connector.core" ] \
	|| { echo "kept a core for a process that is not the frontend" >&2; exit 1; }

setup
run /mnt/sdcard/System/connector "first core"
[ "$(cat "$TMP/card/crash/connector.core")" = "first core" ] \
	|| { echo "the frontend's core was not saved" >&2; exit 1; }

run /mnt/sdcard/System/connector "second core"
[ "$(cat "$TMP/card/crash/connector.core")" = "second core" ] \
	|| { echo "a second crash did not replace the core" >&2; exit 1; }
[ ! -f "$TMP/card/crash/connector.core.tmp" ] \
	|| { echo "left a half-written core behind" >&2; exit 1; }
[ "$(grep -c . "$TMP/card/crash/cores.log")" = "2" ] \
	|| { echo "cores.log did not record both crashes" >&2; exit 1; }

setup
CMDLINE="/lib/ld-linux-aarch64.so.1 /mnt/sdcard/System/connector "
run /lib/ld-linux-aarch64.so.1 "loader-launched core"
[ "$(cat "$TMP/card/crash/connector.core" 2>/dev/null)" = "loader-launched core" ] \
	|| { echo "a frontend started through the loader lost its core" >&2; exit 1; }

setup
CMDLINE="/lib/ld-linux-aarch64.so.1 /usr/bin/connector-helper "
run /lib/ld-linux-aarch64.so.1 "another program"
[ ! -f "$TMP/card/crash/connector.core" ] \
	|| { echo "kept a core for a program whose name only starts like the frontend" >&2; exit 1; }
unset CMDLINE

setup
cat > "$TMP/mountpoint" <<-SH
	#!/bin/sh
	exit 1
SH
chmod +x "$TMP/mountpoint"
run /mnt/sdcard/System/connector "core with nowhere to go" \
	|| { echo "a missing card made the helper fail" >&2; exit 1; }
[ ! -d "$TMP/card/crash" ] \
	|| { echo "wrote to a card that is not mounted" >&2; exit 1; }

echo "coresave: all passed"
