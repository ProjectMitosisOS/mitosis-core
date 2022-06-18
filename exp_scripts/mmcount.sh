#!/bin/bash
pids=$(pgrep -f "simple_child")

echo Detail Logs Begin:
for pid in $pids
do
	echo PID: $pid
	sudo cat /proc/$pid/smaps | grep -i rss | awk '{Total+=$2} END {print Total/1024" MB-RSS"}'
	sudo cat /proc/$pid/smaps | grep -i pss | awk '{Total+=$2} END {print Total/1024" MB-PSS"}'
	echo -n Private_Dirty
	sudo cat /proc/$pid/smaps | grep -i Private_Dirty | awk '{Total+=$2} END {print Total/1024" MB"}'
done

echo Memory Usage Fork End
