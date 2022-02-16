#!/usr/bin/env bash

#target="$1"
target="wxd@val01"
#target = "wxd@cube1"
## this script will sync the project to the remote server
rsync -i -rtuv \
      $PWD/../ \
      $target:~/projects/mos \
      --exclude 'CMakeCache.txt' \
      --exclude 'target' \
      --exclude 'Cargo.lock' \
      --exclude '.git' \
