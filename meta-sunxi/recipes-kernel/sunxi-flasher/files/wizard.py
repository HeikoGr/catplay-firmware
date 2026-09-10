#!/usr/bin/env python3
"""Enter FEL, boot recovery, and flash firmware."""

from __future__ import annotations

import argparse
import os
import shlex
import subprocess
import sys
from pathlib import Path


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--preset", help="Device preset passed to cpbox2fel.py")
    p.add_argument("--fw", default="../v821b-rtl8733bs.c2aflash", help="Local firmware file to flash (default: ../woo.c2aflash)")
    p.add_argument("--already-fel", action="store_true", help="Skip entering FEL")
    p.add_argument("--already-recov", action="store_true", help="Skip entering FEL and booting recovery")
    p.add_argument("--refresh", action="store_true", help="Enter FEL via USB vendor request")
    p.add_argument("--no-flash", action="store_true", help="Only boot recovery")
    args = p.parse_args(argv)
    if not args.already_recov and not args.already_fel and not args.refresh and not args.preset:
        p.error("--preset is required unless --already-recov, --already-fel or --refresh is used")
    return args


def run_step(script: str, argv: list[str]) -> int:
    command = [sys.executable, str(Path(__file__).resolve().parent / script), *argv]
    display_command = [Path(sys.executable).name, os.path.relpath(command[1]), *argv]
    print(f"[*] {shlex.join(display_command)}", flush=True)
    rc = subprocess.run(command).returncode
    if rc != 0:
        print(f"[!] Step failed with exit code {rc}", file=sys.stderr)
    return rc if rc >= 0 else 128 - rc


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    print(r"""
               /\
              /  \
             /____\
            /(o_o )\
           /  /|\   \
          /  / | \   \
             / |\
            /  | \__
           /   |    \__
          /    |       /\_/\
               |      ( o.o )
               |       > ^ <
              / \     /   \
             /   \   /_____\

    THE WIZARD WILL NOW INSTALL YOUR SOFTWARE
    """)
    try:
        if not args.already_recov and not args.already_fel:
            if args.refresh:
                rc = run_step("reboot2recovery.py", ["--mode", "vendor_request"])
            else:
                rc = run_step("cpbox2fel.py", [args.preset])
            if rc != 0:
                return rc

        if not args.already_recov:
            rc = run_step("recov.py", [
                "--fes", "../fes1.bin",
                "--erofs-initrd", "../v821b-rtl8733bs-recov.initrd.bin",
                "--opensbi", "../opensbi.bin",
                "--kernel", "../v821b-rtl8733bs-recov.Image.bin",
                "--extra-bootargs", "c2a_boot=recovery",
                "--simpleboot", "../simpleboot.bin",
                "--dtb", "../woo.dtb",
            ])
            if rc != 0:
                return rc
        if not args.no_flash:
            return run_step("flash.py", ["--host", "192.168.51.2", "--fw", args.fw])
        return 0
    except KeyboardInterrupt:
        print("cancelled", file=sys.stderr)
        return 130
    except OSError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
