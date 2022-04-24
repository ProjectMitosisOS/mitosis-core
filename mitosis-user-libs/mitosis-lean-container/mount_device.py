import os
import argparse
import subprocess

def main():
    parser = argparse.ArgumentParser(description="Mount/Unmount in Rootfs")
    parser.add_argument('--rootfs', required=True)
    parser.add_argument('--device', required=True)
    parser.add_argument('--unmount', action='store_true')
    parser.set_defaults(unmount=False)
    args = parser.parse_args()
    path = os.path.join(args.rootfs, args.device.strip('/'))
    if args.unmount:
        print("unmount device %s from %s" % (args.device, args.rootfs))
        subprocess.run(['umount', path])
    else:
        print("mount device %s to %s" % (args.device, args.rootfs))
        subprocess.run(['touch', path])
        subprocess.run(['mount', '--bind', args.device, path])

if __name__ == '__main__':
    main()
