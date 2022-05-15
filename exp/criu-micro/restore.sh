START=$(date +%s.%N)
echo "before criu restore: $START"
/criu/criu/criu restore --images-dir=./imgs -vvvv -o restore.log
END=$(date +%s.%N)
# TOTAL_TIME=$(echo "($END-$START)*1000" | bc) # uncomment this to calculate the full end-to-end time
