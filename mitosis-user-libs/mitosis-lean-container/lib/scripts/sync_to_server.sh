#!/usr/bin/env bash
# this script will sync the project to the remote server

user="wtx"
target=("val00" "val01" "val02" "val03" "val04" "val05" "val06" "val07" "val08" "val09" "val10" "val11" "val12" "val13" "val14")

for machine in ${target[*]}
do
      rsync -i -rtuv \
            $(find ../build/ -maxdepth 1 -type f -executable) \
            ${user}@${machine}:/home/${user}
done
