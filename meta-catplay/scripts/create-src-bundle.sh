#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <git-repository-root>" >&2
    exit 2
fi

repo_root="$1"
script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
layer_dir="$(cd "${script_dir}/.." && pwd)"
bundle_dir="${layer_dir}/recipes-apps/files"
bundle_path="${bundle_dir}/catplay.tar.gz"

if [ ! -d "${repo_root}/.git" ]; then
    echo "Not a git repository root: ${repo_root}" >&2
    exit 2
fi

mkdir -p "${bundle_dir}"

tar \
    --sort=name \
    --mtime='@0' \
    --owner=0 \
    --group=0 \
    --numeric-owner \
    --pax-option=delete=atime,delete=ctime \
    --exclude='./git' \
    --exclude='.git' \
    --exclude='./.git' \
    --exclude='./firmware' \
    -C "${repo_root}" \
    -cf - . | gzip -n > "${bundle_path}"
