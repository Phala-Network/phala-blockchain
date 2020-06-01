#!/bin/bash

function copy_and_apply {
	src="$1"
	dst="${2:-$1}"
	patch_file="tmp/${dst}.patch"

	if [ ! -f "${patch_file}" ]; then
		echo "Patch file ${patch_file} doesn't exist. Stopping."
		exit -1
	fi

	cp -rT "substrate/bin/node/${src}/" "${dst}/"

	patch -s -p0 < "${patch_file}"
}

copy_and_apply executor
copy_and_apply rpc
copy_and_apply runtime
copy_and_apply cli node
