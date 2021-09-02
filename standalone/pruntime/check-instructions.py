#!/usr/bin/env python3

import os
import sys
import re

try:
    import rust_demangler
except ImportError:
    print("WARN: rust_demangler not installed, skipping demangle", file=sys.stderr)


def error(msg):
    print(msg, file=sys.stderr)


def demangle(name: str):
    try:
        return rust_demangler.demangle(name)
    except:
        return name


class Function:
    def __init__(self, addr, name):
        self.addr = addr
        self.name = name
        self.codes = []
        self.callers = set()
        self.callees = set()


class Checker:
    FUNC_DEF = re.compile(r"([0-9a-f]+) *<(.*)>:")
    CALL_FUNC = re.compile(r"^[^<#]*<(.*)(\+.*)>")

    def __init__(self, filename: str):
        self.filename = filename
        self.functions = self.parse(filename)

    def parse(self, filename) -> 'list[Function]':
        functions = []  # type: list[Function]
        current_function = None
        for line in os.popen(f"objdump -d '{filename}'"):
            match = self.FUNC_DEF.findall(line)
            if match:
                addr, name = match[0]
                current_function = Function(addr, name)
                functions.append(current_function)
            else:
                if current_function:
                    arr = line.split('\t')
                    if len(arr) == 3:
                        _addr, _code, inst = arr
                        current_function.codes.append(inst)
        return functions

    def functions_using(self, op) -> 'set[str]':
        functions = set()
        for func in self.functions:
            for inst in func.codes:
                if inst.split()[0].strip() == op:
                    if func.name not in functions:
                        functions.add(func.name)
        return functions

    def mk_callgraph(self):
        func_map = {
            func.name: func for func in self.functions
        }
        for func in self.functions:
            for inst in func.codes:
                match = self.CALL_FUNC.findall(inst)
                if match:
                    name, _ = match[0]
                    if name != func.name:
                        func.callees.add(name)
                        if name in func_map:
                            func_map[name].callers.add(func.name)
        return func_map

    def function_like(self, name):
        for func in self.functions:
            if name in func.name:
                return func.name
            if name in demangle(func.name):
                return func.name
        return None

    def print_callers(self, func_name, max_depth=4, file=sys.stdout):
        graph = self.mk_callgraph()
        printed = set()

        func = self.function_like(func_name)
        if func is None:
            print(f"No function name match {func_name}", file=file)
            return

        def print_recursive(printed, func, level, max_depth):
            print("  " * level + demangle(func), file=file)
            print("  " * level + func, file=file)
            if func in printed:
                return
            printed.add(func)
            if level >= max_depth:
                return
            for caller in graph[func].callers:
                if caller.startswith("sub_") and len(caller) == 10:
                    continue
                print_recursive(printed, caller, level + 1, max_depth)
        print_recursive(printed, func, 0, max_depth)


if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("filename", nargs="?", help="The path to enclave.so to check", default="./enclave/enclave.so")
    parser.add_argument("-c", "--show-callers", action="store_true", help="Show callers for the ill funtions")
    args = parser.parse_args()

    checker = Checker(args.filename)

    WHITE_LIST = {
        # These funtions are from the intel sgx-sdk. They exist even in no_std.
        # They might be properly handled by the SDK, so we trust Intel that they are safe.
        "cp_is_avx512_extension",
        "cpStopTsc",
        "cpGetCacheSize",
        "cpGetReg",
        "cpStopTscp",
        "cpStartTscp",
        "cp_is_avx_extension",
        "cpStartTsc",
        "cp_get_pentium_counter",
        # TODO.kevin: This one uses SYSCALL.
        #   It is introduced by the teaclave/sgx_unwind, might be a bug?
        "_ULx86_64_sigreturn",
    }
    if os.environ.get("SGX_MODE") == "SW":
        WHITE_LIST.add("_ZL10arch_prctlim")

    ill_instructions = [
        "cpuid",
        "getsec",
        "rdpmc",
        "rdtsc",
        "rdtscp",
        "sgdt",
        "sidt",
        "sldt",
        "str",
        "vmcall",
        "vmfunc",
        "syscall",
        "sysenter",
        "int",
    ]

    ill_functions = []

    for inst in ill_instructions:
        for func in checker.functions_using(inst):
            if demangle(func) in WHITE_LIST:
                continue
            ill_functions.append((func, inst))

    if ill_functions:
        error("="*80)
        error("Error: There are some functions using ILL instructons not allowed in SGX")
        error("")
        for func, inst in ill_functions:
            error(f"{demangle(func)} using {inst}")
        error("")
        error("="*80)

        if args.show_callers:
            error("====== Parsing callers to them =======")
            from callerfinder import CallerFinder
            finder = CallerFinder(args.filename)
            error("="*80)
            error("====== Callers to them =======")
            for func, inst in ill_functions:
                finder.print_callers(demangle(func), 12, sys.stderr)
                error("----------")
            error("="*80)
        error("")
        sys.exit(1)
    else:
        print("Instruction safety check OK")

