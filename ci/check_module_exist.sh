#!/bin/bash

test_machine=$1
username=$2
password=$3
result=$(lsmod | grep testmodule)
if [ "($result)" != "()"  ];then
        echo "Kernel module exists, and we need to reboot the test machine"
        sudo ipmitool -I lanplus -H idrac-$test_machine -U $username -P $password chassis power cycle
else
        echo "Seems like everything is alright"
fi
