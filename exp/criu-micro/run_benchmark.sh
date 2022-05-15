START=$(date +%s.%N)
echo "before start lean container: $START"
../../mitosis-user-libs/mitosis-lean-container/lib/build/start_lean_container $1 $2 /bin/bash /restore.sh
END=$(date +%s.%N)
# TOTAL_TIME=$(echo "($END-$START)*1000" | bc) # uncomment this to calculate the full end-to-end time
tail -n 2 $2/$(pwd)/execution.log
