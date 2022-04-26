IMAGE_NAME=hello
PATH_TO_APP=app/hello/

ROOTFS_DIR=.base/hello/rootfs
ROOTFS_ABS_PATH=${PWD}/${ROOTFS_DIR}

DEVICE=/dev/mitosis-syscalls

export:
	sudo python3 make_app_rootfs.py --app ${PATH_TO_APP} --name ${IMAGE_NAME} --export ${ROOTFS_DIR}


mount_dev:
	sudo python3 mount_device.py --rootfs ${ROOTFS_DIR} --device ${DEVICE}

unmount_dev:
	sudo python3 mount_device.py --rootfs ${ROOTFS_DIR} --device ${DEVICE} --unmount


clean_fs: unmount_dev
	sudo rm -r ${ROOTFS_DIR}

CONTAINER_NAME=my_test_container

COMMAND_ABS_PATH=/usr/local/bin/python
ARGS1=/app/main.py
ARGS2=

run:
	sudo ./lib/build/start_lean_container ${CONTAINER_NAME} ${ROOTFS_ABS_PATH} ${COMMAND_ABS_PATH} ${ARGS1} ${ARGS2}

child:
	sudo ./lib/build/start_lean_container ${CONTAINER_NAME} ${ROOTFS_ABS_PATH} ${COMMAND_ABS_PATH} ${ARGS1} ${ARGS2}
