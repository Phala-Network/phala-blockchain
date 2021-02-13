# SGX

You can install Intel SGX SDK on any Intel CPU (AMD not tested) for only testing purpose. However
the SGX Driver is needed to run the program in hardware mode.

Pass `SGX_MODE=SW make` for simulation and `SGX_MODE=HW make` to build for actual hardware.

## Install SDK

You must install the currect version of the SGX SDK that matches the one used by `rust-sgx-sdk`.

Under the submodule in this repo, you can find a Dockerfile at
`rust-sgx-sdk/dockerfile/Dockerfile.*.nightly`. Choose the one matching your OS.

In the Dockerfile you can find out how the SDK should be installed. Follow the `RUN` commands to
install the SDK.

## Install the SGX Driver

SGX Driver is needed if you are going to run the program in hardware mode. While installing the SDK,
you can find out the base url like:

```
https://download.01.org/intel-sgx/sgx-linux/<version>/distro/<os>/
```

Open it in a browser, and you should be able to locate the url of the driver in the same directory.
(e.g. `sgx_linux_x64_sdk_2.7.101.3.bin`)

Install the driver and reboot. Check if `/dev/isgx` or `lsmod | grep isgx` exists.

**Secure Boot**: It's [reported](https://github.com/intel/linux-sgx-driver/issues/101) that the OS
may fail to load the SGX Driver if Secure Boot is enabled. You need to either disable Secure Boot,
or build the driver and sign it by yourself.

## Check SGX capability

[SGX-hardware](https://github.com/ayeks/SGX-hardware) repo offers a simple C program to check the
CPU capabilities.

## Enable SGX in "Software Control" mode

Some BIOS don't expose a switch of SGX, or only has a "Software Control" mode. In such case, you
can try to try Intel's [sgx-software-enable](https://github.com/intel/sgx-software-enable). Reboot
is needed to take effect.
