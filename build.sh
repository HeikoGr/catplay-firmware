#!/usr/bin/env bash
set -euo pipefail

# This script is meant to be run from outside the catplay-firmware checkout
# (e.g. copied to ~/catplay-build.sh), so which checkout and branch to build
# are explicit inputs rather than derived from the script's own location.
repo_dir="${REPO_DIR:-${HOME}/own/catplay-firmware}"
branch="${BRANCH:-}"

usage() {
    cat <<EOF
Usage: $0 [--repo DIR] [--branch NAME] [bitbake-target] [bitbake-options ...]

Defaults:
  REPO_DIR=${repo_dir}
  MACHINE=${MACHINE:-clk-mini-ultra-nor}
  DISTRO=${DISTRO:-c2a-musl}
  target=${TARGET:-c2a-system-image}

Options:
  --repo DIR      catplay-firmware checkout to build (default: \$REPO_DIR)
  --branch NAME   Branch to check out in --repo before building (default: \$BRANCH, i.e. whatever is currently checked out)

Environment overrides:
  REPO_DIR        Same as --repo
  BRANCH          Same as --branch
  BUILD_DIR       Build directory (default: <REPO_DIR's parent>/catplay-firmware-build)
  CATPLAY_DIR     Local CatPlay checkout (default: <REPO_DIR's parent>/catplay)
  CATPLAY_REPO    CatPlay repository URL
  MACHINE         Yocto machine
  DISTRO          Yocto distro
  TARGET          Default BitBake target
  MIN_BUILD_FREE_GIB  Minimum free space for BUILD_DIR (default: 8)

BUILD_DIR and CATPLAY_DIR default to siblings of REPO_DIR (e.g. both under
~/own/ next to catplay-firmware) rather than inside the checkout, so they
survive branch switches and are shared across every REPO_DIR/BRANCH you
build. This is safe: bitbake's sstate-cache and downloads are keyed by task
content hash, not by branch, so switching branches reuses cached artifacts
for anything that didn't actually change and only rebuilds what did.
tmp-c2a/deploy under BUILD_DIR always reflects the most recently built
MACHINE/DISTRO/branch combination, since auto.conf/bblayers.conf are
regenerated fresh on every run.

Examples:
  $0
  $0 c2a-system-bundle
  $0 --repo ~/own/catplay-firmware --branch build-script c2a-system-bundle
  MACHINE=clk-mini-ultra-nor-recov $0 c2a-system-bundle
  $0 -c cleansstate catplay

Notes on MACHINE and c2a-system-bundle:
  There are two related Carlinkit Mini Ultra machines, both built from the
  same shared.conf/defconfig:
    - clk-mini-ultra-nor       normal firmware image
    - clk-mini-ultra-nor-recov recovery image + host-side flashing tools

  Only clk-mini-ultra-nor-recov's machine conf declares the
  carlinkit-mini-flasher:do_deploy extra dependency for c2a-system-bundle,
  so it's the only one that (re)deploys tools/*.py (exploit.py, flash.py,
  recov.py, ...) into the bundle. If you change anything under
  meta-carlinkit-mini/recipes-bsp/carlinkit-mini-flasher/files/ and want
  those changes reflected in build/tmp-c2a/clk-mini-ultra-nor/tools/, build
  with:
    MACHINE=clk-mini-ultra-nor-recov $0 c2a-system-bundle
  Building c2a-system-bundle with the default MACHINE alone will not pick
  up tools/*.py changes.
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        -h|--help)
            usage
            exit 0
            ;;
        --repo)
            repo_dir="$2"
            shift 2
            ;;
        --repo=*)
            repo_dir="${1#--repo=}"
            shift
            ;;
        --branch)
            branch="$2"
            shift 2
            ;;
        --branch=*)
            branch="${1#--branch=}"
            shift
            ;;
        *)
            break
            ;;
    esac
done

if [[ ! -d "${repo_dir}" ]]; then
    echo "[!] REPO_DIR does not exist: ${repo_dir}" >&2
    exit 1
fi

if [[ "$(git -C "${repo_dir}" rev-parse --show-toplevel 2>/dev/null || true)" != "$(cd "${repo_dir}" && pwd)" ]]; then
    echo "[!] REPO_DIR is not the root of a git checkout: ${repo_dir}" >&2
    exit 1
fi

root_dir="$(cd "${repo_dir}" && pwd)"
own_dir="$(dirname "${root_dir}")"
build_dir="${BUILD_DIR:-${own_dir}/catplay-firmware-build}"
catplay_dir="${CATPLAY_DIR:-${own_dir}/catplay}"
catplay_url="${CATPLAY_REPO:-https://github.com/catplay-labs/catplay.git}"
machine="${MACHINE:-clk-mini-ultra-nor}"
distro="${DISTRO:-c2a-musl}"
target="${TARGET:-c2a-system-image}"

if [[ -n "${branch}" ]]; then
    if [[ -n "$(git -C "${root_dir}" status --porcelain)" ]]; then
        echo "[!] ${root_dir} has uncommitted changes; refusing to switch to branch '${branch}'" >&2
        exit 1
    fi
    echo "[*] Checking out branch ${branch} in ${root_dir}"
    git -C "${root_dir}" fetch origin "${branch}" 2>/dev/null || true
    git -C "${root_dir}" checkout "${branch}"
    if git -C "${root_dir}" rev-parse --abbrev-ref --symbolic-full-name '@{u}' >/dev/null 2>&1; then
        git -C "${root_dir}" merge --ff-only '@{u}' \
            || echo "[!] ${root_dir} has diverged from its upstream; building as-is" >&2
    fi
fi

if [[ $# -gt 0 && "$1" != -* ]]; then
    target="$1"
    shift
fi

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "[!] Required command not found: $1" >&2
        exit 1
    }
}

check_host_dependencies() {
  local tool
  local missing=()
  local host_tools=(
    ar as awk basename bash bunzip2 bzip2 cat chgrp chmod chown chrpath cmp
    comm cp cpio cpp cut date dd diff diffstat dirname du echo egrep env
    expand expr false fgrep file find flock g++ gawk gcc getconf getopt git
    grep gunzip gzip head hostname iconv id install ld ldd ln ls make md5sum
    mkdir mkfifo mknod mktemp mv nm objcopy objdump od patch perl pr printf
    pwd python3 pzstd ranlib readelf readlink realpath rm rmdir rpcgen sed
    seq sh sha1sum sha224sum sha256sum sha384sum sha512sum sleep sort split
    stat strings strip tail tar tee test touch tr true truncate uname uniq
    unzstd wc wget which xargs zstd xz
  )

  for tool in "${host_tools[@]}"; do
    if ! command -v "${tool}" >/dev/null 2>&1; then
      missing+=("${tool}")
    fi
  done

  if [[ ${#missing[@]} -gt 0 ]]; then
    echo "[!] Missing Yocto host tools: ${missing[*]}" >&2
    echo "    Ubuntu/Debian hint: sudo apt-get install build-essential chrpath cpio diffstat file gawk python3 zstd" >&2
    echo "    Install the missing tools and run this script again." >&2
    exit 1
  fi
}

check_python_module() {
  if ! python3 -c "import $1" >/dev/null 2>&1; then
    echo "[!] Missing Python module for Yocto hash equivalence: $1" >&2
    echo "    Install it with: python3 -m pip install --user $1" >&2
    exit 1
  fi
}

check_build_space() {
  local available_kib
  local required_gib="${MIN_BUILD_FREE_GIB:-8}"
  local required_kib

  if ! [[ "${required_gib}" =~ ^[0-9]+$ ]]; then
    echo "[!] MIN_BUILD_FREE_GIB must be a whole number, got: ${required_gib}" >&2
    exit 1
  fi

  mkdir -p "${build_dir}"
  available_kib="$(df -Pk "${build_dir}" | awk 'NR == 2 { print $4 }')"
  required_kib=$((required_gib * 1024 * 1024))

  if [[ -z "${available_kib}" || ${available_kib} -lt ${required_kib} ]]; then
    echo "[!] Insufficient free space for Yocto build directory: ${build_dir}" >&2
    echo "    Required: at least ${required_gib} GiB free; available: $((available_kib / 1024 / 1024)) GiB." >&2
    echo "    Set BUILD_DIR to a larger volume, for example:" >&2
    exit 1
  fi
}

require_command git
require_command tar

echo "[*] Checking build disk space"
check_build_space

echo "[*] Initializing Yocto submodules"
git -C "${root_dir}" submodule sync --recursive
git -C "${root_dir}" submodule update --init --recursive

if [[ ! -f "${root_dir}/openembedded-core/meta/conf/bitbake.conf" ]]; then
  echo "[!] openembedded-core is not initialized correctly" >&2
  exit 1
fi

echo "[*] Checking Yocto host dependencies"
check_host_dependencies
# check_python_module websockets

if [[ ! -d "${catplay_dir}/.git" ]]; then
    mkdir -p "$(dirname "${catplay_dir}")"
    echo "[*] Cloning CatPlay from ${catplay_url}"
    git clone --depth 1 "${catplay_url}" "${catplay_dir}"
elif [[ "$(git -C "${catplay_dir}" rev-parse --show-toplevel)" != "${catplay_dir}" ]]; then
    echo "[!] CATPLAY_DIR is not the root of a CatPlay git checkout: ${catplay_dir}" >&2
    exit 1
fi

echo "[*] Creating CatPlay source bundle"
"${root_dir}/meta-catplay/scripts/create-src-bundle.sh" "${catplay_dir}"

mkdir -p "${build_dir}/conf"

cat > "${build_dir}/conf/bblayers.conf" <<EOF
# Generated by build.sh for '${root_dir}'
LCONF_VERSION = "7"

BBLAYERS = " \\
  ${root_dir}/openembedded-core/meta \\
  ${root_dir}/meta-openembedded/meta-oe \\
  ${root_dir}/meta-openembedded/meta-filesystems \\
  ${root_dir}/meta-openembedded/meta-networking \\
  ${root_dir}/meta-openembedded/meta-multimedia \\
  ${root_dir}/meta-openembedded/meta-python \\
  ${root_dir}/meta-clang \\
  ${root_dir}/meta-freescale \\
  ${root_dir}/meta-sunxi \\
  ${root_dir}/meta-carplay \\
  ${root_dir}/meta-catplay \\
  ${root_dir}/meta-carlinkit \\
  ${root_dir}/meta-carlinkit-mini \\
"
EOF

# Identify which kernel this build produced. This ends up in the kernel banner,
# i.e. the "Linux version ..." line at the top of dmesg, so a captured log says
# what it was built from instead of leaving you to guess whether a flashed
# image contains a given patch.
#
# Deliberately scoped to the paths that actually shape the kernel, not to the
# whole tree: the value becomes a vardep of the kernel's do_compile, so a
# whole-tree "git describe" would rebuild the kernel every time an unrelated
# file (a tool under tools/, a doc) changes. Deliberately derived from git
# rather than the wall clock, for the same reason.
kernel_paths=(
    meta-carlinkit-mini/recipes-letux
    meta-carlinkit-mini/recipes-bsp/carlinkit-mini-defconfig
    meta-carplay/classes
)
kernel_rev="$(git -C "${root_dir}" log -1 --format=%h -- "${kernel_paths[@]}" 2>/dev/null || echo unknown)"
kernel_date="$(git -C "${root_dir}" log -1 --format=%cd --date=format:%Y-%m-%d -- "${kernel_paths[@]}" 2>/dev/null || echo unknown)"
if [[ -n "$(git -C "${root_dir}" status --porcelain -- "${kernel_paths[@]}" 2>/dev/null)" ]]; then
    kernel_rev="${kernel_rev}-dirty"
fi
echo "[*] Kernel source state: ${kernel_rev} (${kernel_date})"

cat > "${build_dir}/conf/auto.conf" <<EOF
# Generated by build.sh for '${root_dir}'
MACHINE = "${machine}"
DISTRO = "${distro}"
LICENSE_FLAGS_ACCEPTED = "commercial"

C2A_KERNEL_REV = "${kernel_rev}"
C2A_KERNEL_DATE = "${kernel_date}"

# ftp.gnu.org (the primary archive) aggressively throttles/rate-limits parallel
# connections, which manifests as do_fetch tasks hanging for a long time.
# Try the ftpmirror.gnu.org redirector (auto-selects a fast nearby mirror)
# *before* falling back to the origin server.
GNU_MIRROR = "https://ftpmirror.gnu.org/gnu"
PREMIRRORS:prepend = " \\
  https://ftp.gnu.org/gnu/ https://ftpmirror.gnu.org/gnu/ \\
  ftp://ftp.gnu.org/gnu/ https://ftpmirror.gnu.org/gnu/ \\
"
MIRRORS:append = " \\
  https://ftp.gnu.org/gnu/ https://ftpmirror.gnu.org/gnu/ \\
"

# Reuse prebuilt Yocto artifacts when available instead of compiling them locally.
# BB_HASHSERVE_UPSTREAM = "wss://hashserv.yoctoproject.org/ws"
# SSTATE_MIRRORS = "file://.* https://sstate.yoctoproject.org/all/PATH;downloadfilename=PATH"
EOF

echo "[*] Starting BitBake: ${target}"
cd "${root_dir}"
# shellcheck disable=SC1091
set +u
source "${root_dir}/openembedded-core/oe-init-build-env" "${build_dir}" >/dev/null
set -u

# BitBake occasionally leaves a stamp pointing at a tmp/deploy/ artifact that
# no longer exists (e.g. after tmp/deploy/ was partially cleaned by hand, or
# a build was interrupted mid-task) - it then trusts the stamp, skips the
# task, and fails much later with a "license-file-missing" QA error or a
# FileNotFoundError copying a missing .ipk into the package feed. Both are
# harmless to retry: clearing the stale stamp forces BitBake to redo a cheap
# packaging step instead of trusting a promise it can't keep. See
# tools/fix-stale-deploy-artifacts.py for details.
max_attempts="${BUILD_RETRY_ATTEMPTS:-3}"
bitbake_log="$(mktemp -t catplay-bitbake-log.XXXXXX)"
trap 'rm -f "${bitbake_log}"' EXIT

attempt=1
while true; do
    set +e
    bitbake "${target}" "$@" 2>&1 | tee "${bitbake_log}"
    bitbake_rc="${PIPESTATUS[0]}"
    set -e

    if [[ "${bitbake_rc}" -eq 0 ]]; then
        exit 0
    fi

    if [[ "${attempt}" -ge "${max_attempts}" ]]; then
        echo "[!] BitBake failed after ${attempt} attempt(s), giving up" >&2
        exit "${bitbake_rc}"
    fi

    if ! python3 "${root_dir}/tools/fix-stale-deploy-artifacts.py" --build-dir "${build_dir}" --from-log "${bitbake_log}"; then
        echo "[!] BitBake failed (exit ${bitbake_rc}) with an error this script doesn't know how to recover from" >&2
        exit "${bitbake_rc}"
    fi

    attempt=$((attempt + 1))
    echo "[*] Retrying BitBake after clearing stale stamps (attempt ${attempt}/${max_attempts})"
done
