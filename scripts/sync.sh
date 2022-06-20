#!/usr/bin/env bash

#target="$1"

#target="wxd@val00"

target=("val01" "val02" "val03" "val04" "val05") ## all the client and server hosts
#target=("val00" "val01") ## all the client and server hosts

#target = "wxd@cube1"
## this script will sync the project to the remote server
for machine in ${target[*]}
do
    rsync -i -rtuv \
          $PWD/../ \
          $machine:~/projects/mos \
          --exclude 'CMakeCache.txt' \
          --exclude 'target' \
          --exclude 'Cargo.lock' \
          --exclude '.git' \

done
