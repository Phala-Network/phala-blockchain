#!/bin/bash

cloc --exclude-dir target,tmp,coverage,node_modules,ring,webpki,substrate,teaclave-sgx-sdk,rizin .
