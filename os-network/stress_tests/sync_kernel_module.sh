#!/usr/bin/env bash

target=("val00" "val01" "val02" "val03" "val04" "val05" "val06" "val07" "val08" "val09") ## all the client and server hosts

#target = "wxd@cube1"
## this script will sync the project to the remote server
for machine in ${target[*]}
do
    rsync -i -rtuv \
          $PWD/rpc_server_tests.ko \
          $PWD/rpc_client_tests.ko \
          $machine:~ \
          --exclude 'CMakeCache.txt' \
          --exclude 'target' \
          --exclude 'Cargo.lock' \
          --exclude '.git' \

done
