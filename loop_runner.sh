#
# /bin/bash

for i in $(seq "$1"); do $2; done

wait
echo 'end of loop runner'