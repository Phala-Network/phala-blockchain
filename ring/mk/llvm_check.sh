#!/usr/bin/bash  


function llvm_version_check() {
  tmpllvm="$(llvm-config --version)" # Check if we can get the llvm version from llvm-config
  if [ $tmpllvm ]
  then
        export llvm_version=$tmpllvm
  fi
}

