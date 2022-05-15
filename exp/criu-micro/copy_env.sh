if [ "$#" -ne 1 ]; then
    echo "Usage: $0 rootfs_path"
    exit 1
fi

# copy images to the rootfs
rm -rf $1/imgs
cp -r imgs $1/imgs

# make directory for the lock file and copy to specific location
rm -rf $1/${PWD}
mkdir -p -m775 $1/${PWD}
cp -r *.py $1/${PWD}/
cp -a -r lock $1/${PWD}/ && echo -n 1 > $1/${PWD}/lock
cp -a -r execution.log $1/${PWD}/

# copy share libraries
cp -r /usr/bin/python3.5 $1/usr/bin/python3.5
cp -r /usr/lib/python3.5/lib-dynload/_ctypes.cpython-35m-x86_64-linux-gnu.so $1/usr/lib/python3.5/lib-dynload/_ctypes.cpython-35m-x86_64-linux-gnu.so
cp -r /usr/lib/python3.5/lib-dynload/mmap.cpython-35m-x86_64-linux-gnu.so $1/usr/lib/python3.5/lib-dynload/mmap.cpython-35m-x86_64-linux-gnu.so
cp -r /usr/lib/locale/locale-archive $1/usr/lib/locale/locale-archive
cp -r /lib/x86_64-linux-gnu/* $1/lib/x86_64-linux-gnu

# copy restore scripts
cp restore.sh $1/restore.sh

# chmod
sudo chmod 666 $1/dev/null
sudo chmod 755 $1
