#!/bin/sh
SD="${AGS_CARD:-/mnt/sdcard}"
MOUNTPOINT="${AGS_MOUNTPOINT:-mountpoint}"
DIR="$SD/crash"

if ! $MOUNTPOINT -q "$SD" 2>/dev/null; then
	cat > /dev/null
	exit 0
fi

exe="${AGS_CORE_EXE:-$(readlink "/proc/$1/exe" 2>/dev/null)}"
# launch_frontend.sh starts the frontend through the dynamic loader, so exe
# names the loader and only the command line names the frontend.
cmd="${AGS_CORE_CMDLINE:-$(tr '\0' ' ' < "/proc/$1/cmdline" 2>/dev/null)}"
mkdir -p "$DIR" 2>/dev/null
echo "$(date '+%F %T') pid=$1 signal=$2 thread=$3 exe=$exe cmd=$cmd" >> "$DIR/cores.log"

case " $exe $cmd " in
*"/connector "*)
	cp "/proc/$1/maps" "$DIR/connector.maps" 2>/dev/null
	cat > "$DIR/connector.core.tmp" && mv -f "$DIR/connector.core.tmp" "$DIR/connector.core"
	;;
*)
	cat > /dev/null
	;;
esac
sync
