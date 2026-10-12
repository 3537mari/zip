# -*- coding: utf-8 -*-
"""Parity: C-bootstrap vs pure-ZIP selfc for small programs."""
from pathlib import Path
import struct
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
ZIPC = ROOT / "bootstrap" / "zipc.exe"
VM = ROOT / "zip-vm" / "target" / "release" / "zip-vm.exe"
SELFC = Path(tempfile.gettempdir()) / "selfc_v2_parity.zipb"
SRC_MAIN = ROOT / "zip-self-zip" / "главное_чистое.zip"


def u(*codes):
    return "".join(chr(c) for c in codes)


def ensure_selfc():
    if SELFC.exists() and SELFC.stat().st_size > 1000:
        return
    v1 = Path(tempfile.gettempdir()) / "selfc_v1_parity.zipb"
    subprocess.check_call([str(ZIPC), str(SRC_MAIN), str(v1)], cwd=ROOT)
    subprocess.check_call([str(VM), str(v1), str(SRC_MAIN), str(SELFC)], cwd=ROOT)


def compile_c(src: Path, out: Path):
    subprocess.check_call([str(ZIPC), str(src), str(out)], cwd=ROOT)


def compile_zip(src: Path, out: Path):
    ensure_selfc()
    subprocess.check_call([str(VM), str(SELFC), str(src), str(out)], cwd=ROOT)


def run_vm(zipb: Path) -> int:
    r = subprocess.run([str(VM), str(zipb)], cwd=ROOT, capture_output=True)
    out = (r.stdout or b"") + (r.stderr or b"")
    text = out.decode("utf-8", "replace")
    for line in text.splitlines():
        if "Результат:" in line or "Pезультат:" in line:
            try:
                return int(line.rsplit(":", 1)[-1].strip())
            except ValueError:
                pass
    # fallback: last integer in output
    import re
    ms = re.findall(r"(-?\d+)", text)
    if ms:
        return int(ms[-1])
    return r.returncode


def nfn(path: Path) -> int:
    data = path.read_bytes()
    return struct.unpack_from("<I", data, 8)[0]


def main():
    fn = u(0x0444, 0x0443, 0x043D, 0x043A, 0x0446, 0x0438, 0x044F)
    gl = u(0x0433, 0x043B, 0x0430, 0x0432, 0x043D, 0x0430, 0x044F)
    vr = u(0x0432, 0x043E, 0x0437, 0x0432, 0x0440, 0x0430, 0x0442)
    tmp = Path(tempfile.gettempdir())
    cases = [
        ("t42", f'#runtime("io")\n{fn} {gl}() {{ {vr}(42) }}\n', 42),
        ("t0", f'#runtime("io")\n{fn} {gl}() {{ {vr}(0) }}\n', 0),
    ]
    ok = 0
    for name, src, expect in cases:
        sp = tmp / f"parity_{name}.zip"
        sp.write_text(src, encoding="utf-8", newline="\n")
        c_out = tmp / f"parity_{name}_c.zipb"
        z_out = tmp / f"parity_{name}_z.zipb"
        compile_c(sp, c_out)
        compile_zip(sp, z_out)
        rc = run_vm(c_out)
        rz = run_vm(z_out)
        magic_c = c_out.read_bytes()[:4]
        magic_z = z_out.read_bytes()[:4]
        print(f"{name}: C result={rc} nfn={nfn(c_out)} ZIP result={rz} nfn={nfn(z_out)} magic={magic_c}/{magic_z}")
        if magic_c == b"ZIPB" and magic_z == b"ZIPB" and rc == expect and rz == expect:
            ok += 1
        else:
            print("  FAIL")
            return 1
    print(f"parity OK {ok}/{len(cases)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
