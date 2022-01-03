#!/usr/bin/env bash

#target="$1"
target="wxd@val01:~/projects/mos"
#target = "wxd@cube1"
## this script will sync the project to the remote server
rsync -i -rtuv \
      $PWD/../ \
      $target \
       --exclude 'network-daemon/target/' \
       --exclude 'mitosis/target' \