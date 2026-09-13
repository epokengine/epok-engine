#!/bin/sh
# Standalone source build: no editor, reflection extractor or host Python needed.
set -eu
build_make=${1:-make}
build_nugget=${2:-third_party/nugget}
build_profile=${3:-Release}
build_folder=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cd "$build_folder"
case "$build_nugget" in
    /*) sdk_root=$build_nugget ;;
    *) sdk_root=$build_folder/$build_nugget ;;
esac
sdk_objects=$(mktemp -d "$sdk_root/psyqo/.epok-sdk-objects.XXXXXXXX")
mkdir -p sdk
sdk_archive_folder=$(mktemp -d "$build_folder/sdk/build.XXXXXXXX")
sdk_archive=${sdk_archive_folder#"$build_folder/"}/libpsyqo.a
sdk_objects=${sdk_objects##*/}
"$build_make" "BUILD=$build_profile" "NUGGET_DIR=$build_nugget" -B epok-standalone-sdk \
    "EPOK_CERTIFIED_SDK=$sdk_archive" "EPOK_SDK_OBJECT_DIRECTORY=$sdk_objects"
"$build_make" "BUILD=$build_profile" "NUGGET_DIR=$build_nugget" -B all \
    "EPOK_CERTIFIED_SDK=$sdk_archive" "EPOK_SDK_OBJECT_DIRECTORY=$sdk_objects"
