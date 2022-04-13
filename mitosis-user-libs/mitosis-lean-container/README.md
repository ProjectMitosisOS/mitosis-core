# Lean Container

## Rootfs Preparation

Suppose you have a directory named `app` and the application python code has put in it with the following structure: 

```
app
└── hello
    ├── hello.py
    ├── requirements.txt
    └── Dockerfile # Optional, we have a default Dockerfile
```

We can build a docker image from `app/hello/`. The name of docker image is `hello`. The rootfs is exported to `.base/hello/rootfs`.

```bash
python3 make_app_rootfs.py --app $PATH_TO_APP$/app/hello/ --name hello --export $OUTPUT_DIR$/$NAME$
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
sudo ./lib/build/test_start_app $OUTPUT_DIR$/$NAME$ hello.py
```
