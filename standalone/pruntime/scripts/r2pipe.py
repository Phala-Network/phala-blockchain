
import os
import time
import json

from subprocess import Popen, PIPE


class R2Pipe:
    def __init__(self, filename):
        radare2_bin = os.environ.get('RADARE2_BIN', 'radare2')
        cmd = [radare2_bin, "-q0", filename]
        try:
            self.process = Popen(
                cmd, shell=False, stdin=PIPE, stdout=PIPE, bufsize=0
            )
            self.process.stdout.read(1)  # initial \x00
        except:
            raise Exception("ERROR: Cannot find radare2 in PATH")

    def cmd(self, cmd):
        cmd = cmd.strip().replace("\n", ";")
        try:
            self.process.stdin.write((cmd + "\n").encode("utf8"))
        except:
            return ''
        r = self.process.stdout
        self.process.stdin.flush()
        out = b""
        foo = None
        while True:
            foo = r.read(1)
            if foo is not None:
                if foo == b"\x00":
                    break
                out += foo
        return out.decode("utf-8", errors="ignore")

    def cmdj(self, cmd):
        return json.loads(self.cmd(cmd))


def open(filename):
    return R2Pipe(filename)
