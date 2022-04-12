# Lean Container

## Image Preparation

```bash
# Prepare for python base image, install pip dependencies here in the rootfs
make python-base-image
# Build the app image and export the image as a rootfs to ${PWD}/.base/
make python-app-image
```

## Running the lean container

See [README.md](./lib/README.md) in lean container C library.
