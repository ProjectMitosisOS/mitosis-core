# Lean Container

## Rootfs Preparation

Place your app in `app/`.

```
app
└── hello
    ├── hello.py
    ├── requirements.txt
    └── Dockerfile # Optional, we have a default Dockerfile
```

We can build a docker image from `app/hello/`. The name of docker image is `hello`. The rootfs is exported to `.base/hello/rootfs`.

```bash
python3 make_app_rootfs.py --app app/hello/ --name hello --export .base/hello/rootfs
```

We can skip the build process of docker image, and only export the docker image.

```bash
python3 make_app_rootfs.py --name hello --only-export .base/hello/rootfs
```

## Running the lean container

Build the lean container.

```bash
cd lib
mkdir build
cd build
cmake ..
make
```

Run the python code in the lean container.

```bash
sudo ./lib/build/test_start_app ./.base/hello/rootfs/ hello.py
```
