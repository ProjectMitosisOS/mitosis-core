fs_path=./testfs

mkdir ${fs_path}

## mount the tmpfs
sudo mount -t tmpfs tmpfs -o size=4G ${fs_path} ;

## generate a test file
head -c 1G </dev/urandom > ./${fs_path}/test


