KMS_DIR=mitosis-kms
KMODULE_NAME=fork

# Build kernel module file
# e.g. make km KMODULE_NAME=fork
km:
	cd ${KMS_DIR} ; python build.py ${KMODULE_NAME}

insmod:
	sudo rmmod ${KMODULE_NAME} ; sudo insmod ${KMS_DIR}/${KMODULE_NAME}.ko

rmmod:
	sudo rmmod ${KMODULE_NAME}

