import os
import argparse
import subprocess

def export_image(image_name, path):
    print('exporting docker image %s to %s' % (image_name, path))
    from pathlib import Path
    Path(path).mkdir(parents=True, exist_ok=True)
    # prepare the docker container
    subprocess.run(['docker', 'rm', '-f', image_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    subprocess.run(['docker', 'create', '--name', image_name, image_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    # export the docker container's rootfs
    container = subprocess.Popen(['docker', 'export', image_name], stdout=subprocess.PIPE)
    subprocess.check_output(['tar', '-C', path, '-xf', '-'], stdin=container.stdout)
    # clean the docker container
    subprocess.run(['docker', 'rm', '-f', image_name], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def build_image(app_path, image_name):
    dockerfile = os.path.join(app_path, 'Dockerfile')
    if not os.path.isfile(dockerfile):
        dockerfile = 'Dockerfile'
    print('build app image %s from %s' % (image_name, app_path))
    subprocess.run(['docker', 'build', '-t', image_name, '-f', dockerfile, app_path])

def main():
    parser = argparse.ArgumentParser(description="Make Application Rootfs")
    parser.add_argument('--only-export')
    parser.add_argument('--export')
    parser.add_argument('--app')
    parser.add_argument('--name')
    args = parser.parse_args()

    if args.name is None:
        print('specify the docker image name by --name')
        return

    if not args.only_export is None:
        print('only export image')
        export_image(args.name, args.only_export)
        return

    if args.app is None:
        print('specify the app directory by --app')
        return
    
    dockerfile = os.path.join(args.app, 'Dockerfile')
    if not os.path.isfile(dockerfile):
        print('cannot find Dockerfile in app directory: %s' % (dockerfile))
        return

    if args.export is None:
        print('specify the export app rootfs directory by --export')
        return
    
    if not os.path.isdir(args.app):
        print('%s is not a directory' % (args.app))
        return

    build_image(args.app, args.name)
    export_image(args.name, args.export)

if __name__ == '__main__':
    main()
