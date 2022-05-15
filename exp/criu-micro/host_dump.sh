if [ "$#" -ne 1 ]; then
    echo "Usage: $0 working_set_size"
    exit 1
fi

echo -n 0 > lock
setsid python3 bench_exe_time_parent.py -working_set $1 -profile 1 < /dev/null > execution.log 2>&1 &
export TARGET_PID=$(pgrep python3)
echo "TARGET_PID=${TARGET_PID}"
rm -rf imgs && mkdir imgs
sleep 3
LD_LIBRARY_PATH=/home/wtx/criu/.deps-install/lib /home/wtx/criu/criu-3.15/criu/criu dump --images-dir=./imgs -t ${TARGET_PID} -vvvv -o dump.log
tail -n 1 imgs/dump.log
