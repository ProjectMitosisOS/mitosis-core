#!/bin/bash

test_machine=$1
username=$2
password=$3
# show debug information
dmesg
result=$(dmesg | grep "kernel BUG")
if [ "($result)" != "()"  ];then
        echo "There is a bug in kernel, and we need to reboot the test machine"
        sudo ipmitool -I lanplus -H idrac-$test_machine -U $username -P $password chassis power cycle
else
        echo "Seems like everything is alright"
fi
