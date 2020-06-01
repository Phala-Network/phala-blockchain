#!/bin/bash

function gen_patch {
	src="$1"
	dst="${2:-$1}"
	diff -Naur -x 'target' "substrate/bin/node/${src}/" "${dst}/" > "tmp/${dst}.patch"
}

gen_patch executor
gen_patch rpc
gen_patch runtime
gen_patch cli node
