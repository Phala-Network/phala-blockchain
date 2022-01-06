#!/usr/bin/bash  


function llvm_version_check() {
  tmpllvm="$(llvm-config --version)" # Check if we can get the llvm version from llvm-config
  if [ $tmpllvm > 10.0.0 ] || [ $tmpllvm > $llvm_version ]; then # Only change llvm_version if newer then version 10 or the local version is newer then the current llvm_version. 
        export llvm_version=$tmpllvm
  fi
}
