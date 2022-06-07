KMS_DIR=mitosis-kms
KMODULE_NAME=fork

# Build kernel module file
# e.g. make km KMODULE_NAME=fork
drop_caches:
	echo 3 | sudo tee /proc/sys/vm/drop_caches

km:
	cd ${KMS_DIR} ; python build.py ${KMODULE_NAME}

insmod:
	sudo rmmod ${KMODULE_NAME} ; sudo insmod ${KMS_DIR}/${KMODULE_NAME}.ko

rmmod:
	sudo rmmod ${KMODULE_NAME}

clean:
	rm -rf ${KMS_DIR}/target

LEAN_APP_NAME=hello

IMAGE_NAME=${LEAN_APP_NAME}
LEAN_CONTAINER_DIR=mitosis-user-libs/mitosis-lean-container
PATH_TO_APP=${LEAN_CONTAINER_DIR}/app/${LEAN_APP_NAME}
ROOTFS_DIR=.base/${IMAGE_NAME}/rootfs
ROOTFS_ABS_PATH=${PWD}/${ROOTFS_DIR}
DEVICE=/dev/mitosis-syscalls


export:
	sudo python3 ${LEAN_CONTAINER_DIR}/make_app_rootfs.py --app ${PATH_TO_APP} --name ${IMAGE_NAME} --export ${ROOTFS_DIR}


mount_dev:
	sudo python3 ${LEAN_CONTAINER_DIR}/mount_device.py --rootfs ${ROOTFS_DIR} --device ${DEVICE}

unmount_dev:
	sudo python3 ${LEAN_CONTAINER_DIR}/mount_device.py --rootfs ${ROOTFS_DIR} --device ${DEVICE} --unmount

clean_fs: unmount_dev
	sudo rm -r ${ROOTFS_DIR}


#CONTAINER_NAME=my_test_container
#COMMAND_ABS_PATH=/usr/local/bin/python
#ARGS1=main.py
#ARGS2=
#ARGS3=

build_lean_lib:
	cd ${LEAN_CONTAINER_DIR}/lib && mkdir -p build && cd build && cmake ../ && make -j


LEAN_BENCH_EXE_PATH=exp/bench_lean_container
BENCH_SEC=5
name=bench_lean_container
command="lean_child"

mac_id=1
handler_id=73

bench_lean_mitosis:
	sudo ${LEAN_BENCH_EXE_PATH} ${BENCH_SEC} ${name} ${ROOTFS_ABS_PATH} ${command} ${mac_id} ${handler_id}

bench_lean_mitosis_bg:
	sudo ${LEAN_BENCH_EXE_PATH} ${BENCH_SEC} ${name} ${ROOTFS_ABS_PATH} ${command} ${mac_id} ${handler_id} &


