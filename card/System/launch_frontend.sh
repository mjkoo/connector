#!/bin/sh
SD="${CONNECTOR_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
RUN="${AGS_RUN:-/run}"
export CONNECTOR_ROOT="$SD"

log() {
	echo "$(date '+%H:%M:%S') $*" >> "$SD/connector-session.log" 2>/dev/null
}

# A card updated from slot. by copying System/ over keeps slot.'s binary and logs.
if [ -f "$SYS/slot" ] && [ -f "$SYS/connector" ]; then
	rm -f "$SYS/slot" "$SD/slot.log" "$SD/slot.log.1" "$SD/slot-session.log" \
		"$SD/crash/slot.core" "$SD/crash/slot.maps"
	log "removed slot.'s binary and logs"
fi

pid="$(cat "$RUN/connector-led.pid" 2>/dev/null)"
if [ -z "$pid" ] || [ ! -d "/proc/$pid" ]; then
	sh "$SYS/led.sh" once
	sh "$SYS/led.sh" loop > /dev/null 2>&1 &
	echo $! > "$RUN/connector-led.pid"
fi

echo "|/bin/sh $SYS/coresave.sh %p %s %e %t" > /proc/sys/kernel/core_pattern 2>/dev/null

[ -f "$SD/connector.log" ] && mv -f "$SD/connector.log" "$SD/connector.log.1"
log "exec $SYS/connector"

exec /lib/ld-linux-aarch64.so.1 "$SYS/connector" > "$SD/connector.log" 2>&1
