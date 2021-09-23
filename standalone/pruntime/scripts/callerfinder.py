import sys
import angr
import rust_demangler


def demangle(name: str):
    try:
        return rust_demangler.demangle(name)
    except:
        return name


class CallerFinder:
    def __init__(self, filename_or_cfg):
        if isinstance(filename_or_cfg, str):
            self.cfg = angr.Project(filename_or_cfg).analyses.CFG()
        else:
            self.cfg = filename_or_cfg

    def callers(self, func):
        predecessors = self.cfg.functions.callgraph.predecessors
        return {self.cfg.functions.function(addr) for addr in predecessors(func.addr)}

    def find(self, name: str):
        result = []
        for func in self.cfg.functions.values():
            if name in demangle(func.name):
                result.append((func, demangle(func.name)))
        return result

    def print_callers(self, func, max_depth=3, file=sys.stdout):
        printed = set()
        if isinstance(func, str):
            funcs = self.find(func)
            if not funcs:
                print(f'No function named "{func}"', file=file)
                return
            else:
                func = funcs[0][0]

        def print_recursive(printed, func, level, max_depth):
            print("  " * level + demangle(func.name), file=file)
            if func in printed:
                return
            printed.add(func)
            if level >= max_depth:
                return
            for caller in self.callers(func):
                if caller.name.startswith("sub_") and len(caller.name) == 10:
                    continue
                print_recursive(printed, caller, level + 1, max_depth)
        print_recursive(printed, func, 0, max_depth)


if __name__ == "__main__":
    finder = CallerFinder('../standalone/pruntime/bin/enclave.signed.so')
    finder.print_callers("std::thread::Thread::new")
