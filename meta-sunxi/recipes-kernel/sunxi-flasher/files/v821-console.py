#!/usr/bin/env python3
"""Attach an interactive terminal to the V821 CDC ACM rescue shell."""

import glob
import os
import select
import sys
import termios
import tty

def find_port():
    for port in sorted(glob.glob("/dev/ttyACM*"), reverse=True):
        node = os.path.basename(port)
        root = os.path.realpath(f"/sys/class/tty/{node}/device")
        while root != "/":
            try:
                with open(os.path.join(root, "idVendor"), encoding="ascii") as f:
                    vendor = f.read().strip()
                with open(os.path.join(root, "idProduct"), encoding="ascii") as f:
                    product = f.read().strip()
                if (vendor, product) == ("0525", "a4a7"):
                    return port
            except OSError:
                pass
            root = os.path.dirname(root)
    raise SystemExit("V821 CDC ACM console (0525:a4a7) not found")


port = find_port()
try:
    fd = os.open(port, os.O_RDWR | os.O_NOCTTY)
except PermissionError:
    raise SystemExit(
        f"No permission for {port}; install config/99-v821-rescue-console.rules"
    )

old_stdin = termios.tcgetattr(sys.stdin.fileno())
port_attr = termios.tcgetattr(fd)
port_attr[3] &= ~(termios.ECHO | termios.ICANON)
termios.tcsetattr(fd, termios.TCSANOW, port_attr)
tty.setraw(sys.stdin.fileno())
print(f"Connected to {port}; Ctrl-] exits\r", flush=True)
last_output_was_cr = False


def write_terminal(data):
    global last_output_was_cr
    out = bytearray()

    for byte in data:
        if byte == 0x0A and not last_output_was_cr:
            out.append(0x0D)
        out.append(byte)
        last_output_was_cr = byte == 0x0D

    os.write(sys.stdout.fileno(), out)

try:
    while True:
        ready, _, _ = select.select([sys.stdin.fileno(), fd], [], [])
        if sys.stdin.fileno() in ready:
            data = os.read(sys.stdin.fileno(), 1024)
            if b"\x1d" in data:
                break
            os.write(fd, data)
        if fd in ready:
            data = os.read(fd, 4096)
            if not data:
                break
            write_terminal(data)
finally:
    termios.tcsetattr(sys.stdin.fileno(), termios.TCSANOW, old_stdin)
    os.close(fd)
