#!/usr/bin/env bash
#
# Script to build the binaries and package them up for release.
#
# shellcheck disable=SC2317

# get the directory of this script and the project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
PROJECT_ROOT="$(git rev-parse --show-toplevel)"

# get shared functions from the volta-install.sh file
# shellcheck source=dev/unix/volta-install.sh
source "${SCRIPT_DIR}/volta-install.sh"

# shellcheck disable=SC2329
usage() {
  cat >&2 <<END_OF_USAGE
release.sh

Compile and package a release for Volta

USAGE:
    ./dev/unix/release.sh [FLAGS] [OPTIONS]

FLAGS:
    -h, --help          Prints this help info

OPTIONS:
        --release       Build artifacts in release mode, with optimizations (default)
        --dev           Build artifacts in dev mode, without optimizations
END_OF_USAGE
}


# default to compiling with '--release'
build_with_release="true"

# parse input arguments
case "$1" in
  -h|--help)
    usage
    exit 0
    ;;
  --dev)
    build_with_release="false"
    ;;
  ''|--release)
    # not really necessary to set this again
    build_with_release="true"
    ;;
  *)
    error "Unknown argument '$1'"
    usage
    exit1
    ;;
esac

# read the current version from Cargo.toml
cargo_toml_contents="$(<"${PROJECT_ROOT}/Cargo.toml")"
VOLTA_VERSION="$(parse_cargo_version "${cargo_toml_contents}")" || exit 1

# figure out the OS details
os="$(uname -s)"
openssl_version="$(openssl version)" || exit 1
if ! VOLTA_OS="$(parse_os_info "${os}" "${openssl_version}")"; then
  error "Releases for '${os}' are not yet supported."
  request "To support '${os}', add another case to parse_os_info() in volta-install.sh."
  exit 1
fi

release_filename="volta-${VOLTA_VERSION}-${VOLTA_OS}"

# first make sure the release binaries have been built
bold_filename="$(bold "${release_filename}")"
info 'Building' "Volta for ${bold_filename}"
if [[ "${build_with_release}" == "true" ]]
then
  target_dir="${PROJECT_ROOT}/target/release"
  cargo build --release --manifest-path="${PROJECT_ROOT}/Cargo.toml"
else
  target_dir="${PROJECT_ROOT}/target/debug"
  cargo build --manifest-path="${PROJECT_ROOT}/Cargo.toml"
fi || exit 1

# then package the binaries and shell scripts together
info 'Packaging' "the compiled binaries"
# using COPYFILE_DISABLE to avoid storing extended attribute files when run on OSX
# (see https://superuser.com/q/61185)
COPYFILE_DISABLE=1 tar -czvf "${target_dir}/${release_filename}.tar.gz" -C "${target_dir}" volta volta-shim volta-migrate

info 'Completed' "release in file ${target_dir}/${release_filename}.tar.gz"
