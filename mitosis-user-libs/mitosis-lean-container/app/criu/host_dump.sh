echo -n 0 > lock
setsid python3 benchmark.py -run_sec 10 -lock_string 0 -working_set 16777216 -lock_file lock  -lock_string 0 < /dev/null > /dev/null 2>&1 &
export TARGET_PID=$(pgrep python3)
echo "TARGET_PID=${TARGET_PID}"
rm -rf imgs && mkdir imgs
LD_LIBRARY_PATH=/home/wtx/criu/.deps-install/lib /home/wtx/criu/criu-3.15/criu/criu dump --images-dir=./imgs -t ${TARGET_PID} -vvvv -o dump.log
