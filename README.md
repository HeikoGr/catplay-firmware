# Yocto layer for CatPlay (Carplay2Air)
  
Yocto version: wrynose
Dependencies:
- meta-clang@wrynose
- meta-freescale@wrynose
- meta-openembedded@wrynose
- openembedded-core@wrynose
- bitbake@yocto-6.0.2

## Build

The repository contains a reproducible build entry point for the Carlinkit Mini
Ultra NOR image. It initializes the Yocto submodules, clones CatPlay into
`.sources/catplay`, creates the source bundle expected by `meta-catplay`, and
starts BitBake:

```sh
./build.sh
```

The default machine is `clk-mini-ultra-nor`, the distro is `c2a-musl`, and the
target is `c2a-system-image`. The generated artifacts are placed below
`build/tmp/deploy/images/clk-mini-ultra-nor/`. The machine-specific `.c2aflash`
image can be selected explicitly with:

```sh
./build.sh c2a-system-bundle
```

Set `MACHINE`, `DISTRO`, `TARGET`, `BUILD_DIR`, or `CATPLAY_DIR` in the
environment to override the defaults. The script uses the official GNU archive
with the geo mirror as a fallback and enables Yocto's public
sstate cache; unavailable cache objects are built locally.

The hash-equivalence cache requires the Python `websockets` module. On a new
development container, install it for the active Python interpreter with:

```sh
python3 -m pip install --user websockets
```

Yocto requires substantial temporary storage. Keep at least 8 GiB free on the
volume containing `BUILD_DIR`; the default is `build/`. To retain the current
build state while moving it to a larger volume, copy it first and only remove
the original after a successful build:

```sh
rsync -aHAX --info=progress2 build/ /tmp/catplay-build/
BUILD_DIR=/tmp/catplay-build ./build.sh
```
