#!/usr/bin/env python3
import re
import sys
import r2pipe


def error(msg):
    print(msg, file=sys.stderr)


class Checker:
    SYSCALL_WHITELIST = [
        re.compile(r)
        for r in [
            # We have no way to patch the syscalls used in std::sync::Once unless to patch the std itself.
            # The `Once` uses thread::park() which futher calls futex.
            # Since we let the futex syscall return immediately, the logic in `Once` is equivalent to using a spinlock.
            r"sym.std::sync::once::Once::call_inner::h[0-9a-f]+",
            r"sym._std::sync::once::WaiterQueue_as_core::ops::drop::Drop_::drop::h[0-9a-f]+",
        ]
    ]

    def __init__(self, filename):
        self.p = r2pipe.open(filename)
        self.p.cmd("e bin.cache=true;aa;aae;")  # run analysis

    def check_syscalls(self):
        p = self.p
        supported_syscalls = {
            213,  # SYS_epoll_create
            283,  # SYS_timerfd_create
            286,  # SYS_timerfd_settime
            287,  # SYS_timerfd_gettime
            291,  # SYS_epoll_create1
            318,  # SYS_getrandom
            332,  # SYS_statx
        }

        invalid_syscalls = []
        for ref in p.cmdj("axtj @sym.syscall"):
            if ref["type"].lower() != "call":
                continue
            ins_addr = ref["from"]
            fn_info = p.cmdj("pdfj @ 0x%x" % ins_addr)
            p0 = self.get_arg(ins_addr, 0, fn_info=fn_info)
            if p0 not in supported_syscalls and not self.is_white(fn_info["name"]):
                invalid_syscalls.append((ins_addr, fn_info["name"], p0))
        return invalid_syscalls

    def is_white(self, name):
        for regex in self.SYSCALL_WHITELIST:
            if regex.match(name.strip()):
                return True
        return False

    def get_arg(self, ins_addr, ind, fn_info=None):
        fn_info = fn_info or self.p.cmdj("pdfj @ 0x%x" % ins_addr)
        ops = fn_info["ops"]
        reg = ["edi"][ind]

        # seek to the call instruction position
        for i, op in enumerate(ops):
            if op["offset"] == ins_addr:
                break
        else:
            raise Exception("Could not find instruction at 0x%x" % ins_addr)

        # seek back for the first argument
        for op in reversed(ops[:i]):
            if reg in op["opcode"]:
                opcode = op["opcode"]
                if "xor {0}, {0}".format(reg) == opcode:
                    p0 = 0
                    break
                elif opcode.startswith("mov {},".format(reg)):
                    p0 = eval(opcode.split(",")[1])
                    break
        else:
            error(self.p.cmdj("pdf @ 0x%x" % ins_addr))
            raise Exception("Could not find parameter for syscall at 0x%x" % ins_addr)
        return p0


def main():
    filename = sys.argv[1:] or ["enclave/enclave.so"]
    checker = Checker(filename[0])
    inv_calls = checker.check_syscalls()

    if inv_calls:
        error("=" * 40)
        error("Found unsupported syscalls:")
        for addr, name, p0 in inv_calls:
            error("Calling (%d) at 0x%x in %s" % (p0, addr, name))
        error("=" * 40)
        sys.exit(1)


if __name__ == "__main__":
    main()
