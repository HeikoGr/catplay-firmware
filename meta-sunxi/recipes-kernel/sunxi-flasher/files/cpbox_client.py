"""Internal CPBox API for OTA downloads, repacking, and device updates."""

from __future__ import annotations

import dataclasses
import hashlib
import json
import os
import re
import shutil
import struct
import subprocess
import tempfile
import time
import urllib.parse
import urllib.request
import uuid
from pathlib import Path
from typing import Any


class CPBoxOtaClient:
    """Download an OTA published for a CPBox LY project."""

    _OTA_HOST = "cpbox-abroad.oss-us-west-1.aliyuncs.com"
    _METADATA_FILENAME = "version.json"
    _UPDATE_FILENAME = "update.img"
    _MAX_METADATA_SIZE = 1024 * 1024
    _MAX_UPDATE_SIZE = 512 * 1024 * 1024
    _CHUNK_SIZE = 1024 * 1024
    _LY_PATTERN = re.compile(r"ly([0-9]{4})")
    _DECRYPT_METHOD_BY_LY = {"5166": "v821", "5190": "v821"}
    _USER_AGENT = "Mozilla/5.0 (iPhone; CPU iPhone OS 18_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 Version/26.0 Safari/605.1.15"

    class Error(RuntimeError):
        """Raised when OTA metadata or content cannot be downloaded safely."""

    @dataclasses.dataclass(frozen=True)
    class Download:
        update_path: Path
        decrypt_method: str

    def __init__(self, *, timeout: int = 60) -> None:
        if timeout <= 0:
            raise ValueError("timeout must be positive")
        self._timeout = timeout

    def download_ota(self, ly: str, output_dir: Path | str) -> Download:
        """Download metadata and its referenced update for e.g. ``ly5190``.

        The raw metadata is saved as ``version.json`` before the update is
        downloaded atomically to ``update.img``. The result also carries the
        decryption method assigned to the LY project.
        """
        ly_number = self._ly_number(ly)
        decrypt_method = self.decrypt_method_for(ly)
        destination = Path(output_dir)
        destination.mkdir(parents=True, exist_ok=True)
        if not destination.is_dir():
            raise self.Error(f"output path is not a directory: {destination}")

        metadata_url = f"https://{self._OTA_HOST}/{ly_number}/{self._METADATA_FILENAME}"
        metadata_raw = self._fetch_bytes(metadata_url, self._MAX_METADATA_SIZE)
        metadata = self._parse_metadata(metadata_raw, metadata_url)
        update_url = self._validated_update_url(metadata.get("url"), ly_number)

        self._write_atomically(destination / self._METADATA_FILENAME, metadata_raw)
        update_path = destination / self._UPDATE_FILENAME
        self._download_atomically(update_url, update_path, self._MAX_UPDATE_SIZE)
        return self.Download(update_path=update_path, decrypt_method=decrypt_method)

    def decrypt_method_for(self, ly: str) -> str:
        """Return the encryption method for a downloaded or cached OTA."""
        ly_number = self._ly_number(ly)
        try:
            return self._DECRYPT_METHOD_BY_LY[ly_number]
        except KeyError as error:
            raise self.Error(f"no decryption method configured for {ly}") from error

    def _ly_number(self, ly: str) -> str:
        if not isinstance(ly, str):
            raise self.Error("LY project must be a string in the lyXXXX format")
        match = self._LY_PATTERN.fullmatch(ly)
        if match is None:
            raise self.Error(f"invalid LY project {ly!r}; expected lyXXXX")
        return match.group(1)

    def _request(self, url: str) -> urllib.request.Request:
        return urllib.request.Request(url, headers={"User-Agent": self._USER_AGENT})

    def _fetch_bytes(self, url: str, limit: int) -> bytes:
        try:
            with urllib.request.urlopen(self._request(url), timeout=self._timeout) as response:
                self._check_content_length(response.headers.get("Content-Length"), limit, url)
                content = response.read(limit + 1)
        except (OSError, ValueError) as error:
            raise self.Error(f"cannot download {url}: {error}") from error
        if len(content) > limit:
            raise self.Error(f"response exceeds the {limit}-byte limit: {url}")
        return content

    def _parse_metadata(self, raw: bytes, url: str) -> dict[str, Any]:
        try:
            metadata = json.loads(raw.decode("utf-8-sig"))
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise self.Error(f"invalid OTA metadata from {url}: {error}") from error
        if not isinstance(metadata, dict):
            raise self.Error(f"OTA metadata from {url} is not a JSON object")
        return metadata

    def _validated_update_url(self, value: object, ly_number: str) -> str:
        if not isinstance(value, str):
            raise self.Error("OTA metadata does not contain a string URL")
        parsed = urllib.parse.urlsplit(value)
        if (
            parsed.scheme != "https"
            or parsed.hostname != self._OTA_HOST
            or parsed.username is not None
            or parsed.password is not None
            or not parsed.path.startswith(f"/{ly_number}/")
        ):
            raise self.Error(
                f"refusing OTA URL outside the LY {ly_number} directory on {self._OTA_HOST}"
            )
        return value

    def _download_atomically(self, url: str, target: Path, limit: int) -> None:
        temporary_path: Path | None = None
        try:
            with urllib.request.urlopen(self._request(url), timeout=self._timeout) as response:
                self._check_content_length(response.headers.get("Content-Length"), limit, url)
                with tempfile.NamedTemporaryFile(
                    mode="wb",
                    dir=target.parent,
                    prefix=f".{target.name}.",
                    suffix=".part",
                    delete=False,
                ) as output:
                    temporary_path = Path(output.name)
                    size = 0
                    while chunk := response.read(self._CHUNK_SIZE):
                        size += len(chunk)
                        if size > limit:
                            raise self.Error(f"response exceeds the {limit}-byte limit: {url}")
                        output.write(chunk)
                    output.flush()
                    os.fsync(output.fileno())
            temporary_path.replace(target)
        except self.Error:
            raise
        except (OSError, ValueError) as error:
            raise self.Error(f"cannot download {url}: {error}") from error
        finally:
            if temporary_path is not None:
                temporary_path.unlink(missing_ok=True)

    def _write_atomically(self, target: Path, content: bytes) -> None:
        temporary_path: Path | None = None
        try:
            with tempfile.NamedTemporaryFile(
                mode="wb",
                dir=target.parent,
                prefix=f".{target.name}.",
                suffix=".part",
                delete=False,
            ) as output:
                temporary_path = Path(output.name)
                output.write(content)
                output.flush()
                os.fsync(output.fileno())
            temporary_path.replace(target)
        except OSError as error:
            raise self.Error(f"cannot save {target}: {error}") from error
        finally:
            if temporary_path is not None:
                temporary_path.unlink(missing_ok=True)

    def _check_content_length(self, value: str | None, limit: int, url: str) -> None:
        if value is None:
            return
        try:
            declared_size = int(value)
        except ValueError as error:
            raise self.Error(f"invalid Content-Length returned for {url}") from error
        if declared_size < 0 or declared_size > limit:
            raise self.Error(f"response exceeds the {limit}-byte limit: {url}")


class CPBoxDeviceClient:
    """Query a CPBox and submit a prepared OTA to its local updater."""

    _APPVER_PATTERN = re.compile(r"^(\d+)\.(\d+)\.(\d+)$")
    _HOST_ENDPOINT = "/cgi-bin/index.cgi?id=host"
    _UPLOAD_ENDPOINT = "/cgi-bin/index.cgi?id=upload"
    _REBOOT_ENDPOINT = "/cgi-bin/index.cgi?id=set&conn=cp"
    _MAX_RESPONSE_SIZE = 1024 * 1024
    _MAX_UPLOAD_SIZE = 512 * 1024 * 1024
    _POLL_INTERVAL = 0.5
    _USER_AGENT = "catplay-v821-fel-flow/0.2"

    class Error(RuntimeError):
        """Raised when the device cannot be queried or updated safely."""

    @dataclasses.dataclass(frozen=True)
    class Device:
        ly: str
        build: int
        appver: str
        sid: object
        name: object
        update: int
        raw: dict[str, Any]

    def __init__(self, device_url: str, *, timeout: int = 30) -> None:
        if timeout <= 0:
            raise ValueError("timeout must be positive")
        self._device_url = self._normalize_url(device_url)
        self._timeout = timeout

    def query_device(self) -> Device:
        value = self._request_json(f"{self._device_url}{self._HOST_ENDPOINT}")
        sys_info = value.get("sys")
        if not isinstance(sys_info, dict):
            raise self.Error("device host response has no sys object")
        appver = sys_info.get("appver")
        if not isinstance(appver, str):
            raise self.Error("device appver is not a string")
        match = self._APPVER_PATTERN.fullmatch(appver)
        if match is None:
            raise self.Error(f"unexpected device appver format: {appver!r}")
        update = value.get("update")
        if not isinstance(update, int) or isinstance(update, bool):
            raise self.Error(f"device update state is not an integer: {update!r}")
        return self.Device(
            ly=f"ly{match.group(2)}",
            build=int(match.group(1)),
            appver=appver,
            sid=value.get("sn"),
            name=value.get("name"),
            update=update,
            raw=value,
        )

    def upload(self, image: bytes | Path | str) -> dict[str, Any]:
        if isinstance(image, (Path, str)):
            image_path = Path(image)
            if not image_path.exists():
                raise self.Error(f"patched OTA does not exist: {image_path}")
            if not image_path.is_file() or image_path.is_symlink():
                raise self.Error(f"patched OTA is not a regular file: {image_path}")
            try:
                image_size = image_path.stat().st_size
            except OSError as error:
                raise self.Error(f"cannot inspect patched OTA {image_path}: {error}") from error
            if image_size > self._MAX_UPLOAD_SIZE:
                raise self.Error(f"OTA exceeds the {self._MAX_UPLOAD_SIZE}-byte upload limit")
            try:
                image = image_path.read_bytes()
            except OSError as error:
                raise self.Error(f"cannot read patched OTA {image_path}: {error}") from error
        elif not isinstance(image, bytes):
            raise self.Error("OTA upload must be bytes or a file path")
        if not image:
            raise self.Error("refusing to upload an empty OTA")
        if len(image) > self._MAX_UPLOAD_SIZE:
            raise self.Error(f"OTA exceeds the {self._MAX_UPLOAD_SIZE}-byte upload limit")
        boundary = f"----catplay-{uuid.uuid4().hex}"
        body = bytearray()
        body += f"--{boundary}\r\n".encode("ascii")
        body += b'Content-Disposition: form-data; name="file"; filename="blob"\r\n'
        body += b"Content-Type: application/octet-stream\r\n\r\n"
        body += image
        body += f"\r\n--{boundary}--\r\n".encode("ascii")
        headers = {
            "Content-Type": f"multipart/form-data; boundary={boundary}",
            "Content-Length": str(len(body)),
        }
        response = self._open(
            self._request(
                f"{self._device_url}{self._UPLOAD_ENDPOINT}",
                data=bytes(body),
                headers=headers,
            ),
            timeout=max(self._timeout, 60),
        )
        return self._decode_success_response(response, "upload")

    def wait_for_update_complete(self, timeout: int = 600) -> Device:
        if timeout <= 0:
            raise ValueError("timeout must be positive")
        deadline = time.monotonic() + timeout
        last_state: object = None
        while time.monotonic() < deadline:
            device = self.query_device()
            last_state = device.update
            if device.update == 3:
                return device
            if device.update > 3:
                raise self.Error(f"device update failed with state {device.update}")
            time.sleep(self._POLL_INTERVAL)
        raise self.Error(
            f"timed out after {timeout}s waiting for update state 3 "
            f"(last update state={last_state!r})"
        )

    def reboot(self) -> dict[str, Any]:
        response = self._open(
            self._request(f"{self._device_url}{self._REBOOT_ENDPOINT}"),
            timeout=self._timeout,
        )
        return self._decode_success_response(response, "reboot")

    def _request(
        self,
        url: str,
        *,
        data: bytes | None = None,
        headers: dict[str, str] | None = None,
    ) -> urllib.request.Request:
        request_headers = {"User-Agent": self._USER_AGENT}
        if headers:
            request_headers.update(headers)
        return urllib.request.Request(url, data=data, headers=request_headers)

    def _request_json(self, url: str) -> dict[str, Any]:
        response = self._open(self._request(url), timeout=self._timeout)
        return self._decode_json(response, url)

    def _open(self, request: urllib.request.Request, *, timeout: int) -> bytes:
        try:
            with urllib.request.urlopen(request, timeout=timeout) as response:
                status = getattr(response, "status", 200)
                if status != 200:
                    raise self.Error(f"device returned HTTP {status} for {request.full_url}")
                declared = response.headers.get("Content-Length")
                if declared is not None:
                    try:
                        declared_size = int(declared)
                    except ValueError as error:
                        raise self.Error(
                            f"invalid Content-Length returned by {request.full_url}"
                        ) from error
                    if declared_size < 0 or declared_size > self._MAX_RESPONSE_SIZE:
                        raise self.Error(f"device response is too large: {request.full_url}")
                raw = response.read(self._MAX_RESPONSE_SIZE + 1)
        except self.Error:
            raise
        except (OSError, ValueError) as error:
            raise self.Error(f"device request failed for {request.full_url}: {error}") from error
        if len(raw) > self._MAX_RESPONSE_SIZE:
            raise self.Error(f"device response is too large: {request.full_url}")
        return raw

    def _decode_json(self, raw: bytes, source: str) -> dict[str, Any]:
        try:
            value = json.loads(raw)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise self.Error(f"invalid JSON from {source}: {error}") from error
        if not isinstance(value, dict):
            raise self.Error(f"expected a JSON object from {source}")
        return value

    def _decode_success_response(self, raw: bytes, operation: str) -> dict[str, Any]:
        value = self._decode_json(raw, operation)
        if value.get("code") != 0:
            raise self.Error(f"{operation} failed: {value!r}")
        return value

    def _normalize_url(self, value: str) -> str:
        parsed = urllib.parse.urlsplit(value)
        if (
            parsed.scheme != "http"
            or not parsed.hostname
            or parsed.username is not None
            or parsed.password is not None
        ):
            raise self.Error("device URL must be a plain http:// host URL")
        if parsed.path not in ("", "/") or parsed.query or parsed.fragment:
            raise self.Error("device URL must not contain a path, query, or fragment")
        return urllib.parse.urlunsplit((parsed.scheme, parsed.netloc, "", "", ""))


class UpdateImageEnvelope:
    """Validate and rebuild the vendor MD5/update.swu envelope."""

    _HEADER_PATTERN = re.compile(rb"^([0-9a-fA-F]{32})  update\.swu\n")

    class Error(RuntimeError):
        """Raised when the vendor update envelope is invalid."""

    def unpack(self, content: bytes) -> bytes:
        match = self._HEADER_PATTERN.match(content[:256])
        if match is None:
            raise self.Error("decrypted image lacks the vendor MD5/update.swu header")
        archive = content[match.end() :]
        expected = match.group(1).decode("ascii").lower()
        actual = hashlib.md5(archive).hexdigest()  # nosec: required by vendor format
        if actual != expected:
            raise self.Error(f"SWU MD5 mismatch: expected {expected}, got {actual}")
        return archive

    def repack(self, archive: bytes) -> bytes:
        digest = hashlib.md5(archive).hexdigest()  # nosec: required by vendor format
        return digest.encode("ascii") + b"  update.swu\n" + archive


class CpioArchive:
    """Parse and rebuild a newc/CRC CPIO archive."""

    _ALIGNMENT = 4096
    _HEADER_SIZE = 110
    _MAGIC = {b"070701", b"070702"}
    _CRC_MAGIC = b"070702"
    _TRAILER = "TRAILER!!!"

    class Error(RuntimeError):
        """Raised when an SWU CPIO archive violates its invariants."""

    @dataclasses.dataclass
    class Entry:
        magic: bytes
        name: str
        fields: list[int]
        data: bytes

    def __init__(self, entries: list[Entry]) -> None:
        self._entries = entries

    @classmethod
    def parse(cls, raw: bytes) -> CpioArchive:
        entries: list[CpioArchive.Entry] = []
        offset = 0
        while True:
            if offset + cls._HEADER_SIZE > len(raw):
                raise cls.Error("truncated CPIO header")
            magic = raw[offset : offset + 6]
            if magic not in cls._MAGIC:
                raise cls.Error(f"invalid CPIO magic at offset {offset}: {magic!r}")
            try:
                fields = [
                    int(raw[offset + 6 + index * 8 : offset + 14 + index * 8], 16)
                    for index in range(13)
                ]
            except ValueError as error:
                raise cls.Error(f"invalid CPIO header at offset {offset}") from error

            name_size = fields[11]
            file_size = fields[6]
            if name_size < 1:
                raise cls.Error("CPIO member has an empty name field")
            name_start = offset + cls._HEADER_SIZE
            name_end = name_start + name_size
            if name_end > len(raw) or raw[name_end - 1] != 0:
                raise cls.Error("truncated or unterminated CPIO member name")
            try:
                name = raw[name_start : name_end - 1].decode("utf-8")
            except UnicodeDecodeError as error:
                raise cls.Error("non-UTF-8 CPIO member name") from error

            data_start = (name_end + 3) & ~3
            data_end = data_start + file_size
            if data_end > len(raw):
                raise cls.Error(f"truncated CPIO data for {name}")
            data = raw[data_start:data_end]
            if magic == cls._CRC_MAGIC and (sum(data) & 0xFFFFFFFF) != fields[12]:
                raise cls.Error(f"CPIO CRC mismatch for {name}")
            entries.append(cls.Entry(magic=magic, name=name, fields=fields, data=data))
            offset = (data_end + 3) & ~3
            if name == cls._TRAILER:
                break

        if any(raw[offset:]):
            raise cls.Error("non-zero bytes follow the CPIO trailer")
        if len(raw) % cls._ALIGNMENT:
            raise cls.Error(f"SWU archive is not padded to {cls._ALIGNMENT} bytes")
        names = [entry.name for entry in entries]
        if not names or names[0] != "sw-description" or names[-1] != cls._TRAILER:
            raise cls.Error("unexpected CPIO member ordering")
        if len(names) != len(set(names)):
            raise cls.Error("duplicate CPIO member names")
        return cls(entries)

    @property
    def names(self) -> tuple[str, ...]:
        return tuple(entry.name for entry in self._entries)

    def member(self, name: str) -> bytes:
        matches = [entry.data for entry in self._entries if entry.name == name]
        if len(matches) != 1:
            raise self.Error(f"expected exactly one CPIO member named {name!r}")
        return matches[0]

    def replace_member(self, name: str, content: bytes) -> None:
        indexes = [index for index, entry in enumerate(self._entries) if entry.name == name]
        if len(indexes) != 1:
            raise self.Error(f"expected exactly one CPIO member named {name!r}")
        index = indexes[0]
        self._entries[index] = dataclasses.replace(self._entries[index], data=content)

    def repack(self) -> bytes:
        result = bytearray()
        for entry in self._entries:
            name = entry.name.encode("utf-8") + b"\0"
            fields = list(entry.fields)
            fields[6] = len(entry.data)
            fields[11] = len(name)
            fields[12] = (
                sum(entry.data) & 0xFFFFFFFF if entry.magic == self._CRC_MAGIC else 0
            )
            result += entry.magic
            result += b"".join(f"{value:08X}".encode("ascii") for value in fields)
            result += name
            result += b"\0" * (-len(result) % 4)
            result += entry.data
            result += b"\0" * (-len(result) % 4)
        result += b"\0" * (-len(result) % self._ALIGNMENT)
        return bytes(result)


class BootScriptPatcher:
    """Inject the guarded V821 FEL hook into app/lyLink.sh."""

    _HOOK_MARKER = b"CATPLAY-V821-FEL-HOOK-v1"
    _FEL_PAYLOAD = (
        b"echo 0x4A000208 0x5aa5a55a > /sys/class/sunxi_dump/write; "
        b"sync; reboot -f; sleep 30\n"
    )
    _RETRY_WRAPPER_PREFIX = b"""# CATPLAY-V821-FEL-HOOK-v1 BEGIN
# Five bounded attempts to enter BootROM FEL after the vendor OTA has finished.
# This deliberately does not touch UDISK/update_flag, ly_boot_mode,
# boot_partition, swu_mode, or swu_next.
catplay_v821_fel_hook() {
    catplay_fel_name=catplay_fel_attempts
    catplay_fel_done_name=catplay_fel_complete
    catplay_fel_done=$(fw_printenv -n "$catplay_fel_done_name" 2>/dev/null || true)
    if [ "$catplay_fel_done" = 1 ]; then
        fw_setenv "$catplay_fel_name" >/dev/null 2>&1 || true
        return
    fi

    catplay_fel_tries=$(fw_printenv -n "$catplay_fel_name" 2>/dev/null || true)
    if [ -z "$catplay_fel_tries" ]; then
        catplay_fel_tries=5
    else
        case "$catplay_fel_tries" in
            *[!0-9]*)
                fw_setenv "$catplay_fel_done_name" 1 >/dev/null 2>&1 || true
                fw_setenv "$catplay_fel_name" >/dev/null 2>&1 || true
                return
                ;;
        esac
    fi

    if [ "$catplay_fel_tries" -le 0 ]; then
        fw_setenv "$catplay_fel_done_name" 1 >/dev/null 2>&1 || true
        fw_setenv "$catplay_fel_name" >/dev/null 2>&1 || true
        return
    fi

    catplay_fel_next=$((catplay_fel_tries - 1))
    if ! fw_setenv "$catplay_fel_name" "$catplay_fel_next"; then
        echo "CatPlay FEL: cannot save retry counter; continuing stock boot" >&2
        return
    fi

    echo "CatPlay FEL: requesting BootROM FEL, attempts left=$catplay_fel_next"
"""
    _RETRY_WRAPPER_SUFFIX = b"""}
catplay_v821_fel_hook
unset -f catplay_v821_fel_hook 2>/dev/null || true
unset catplay_fel_name catplay_fel_done_name catplay_fel_done
unset catplay_fel_tries catplay_fel_next
# CATPLAY-V821-FEL-HOOK-v1 END

"""

    class Error(RuntimeError):
        """Raised when lyLink.sh cannot be patched safely."""

    def build_payload(self) -> bytes:
        return self._FEL_PAYLOAD

    def build_retry_wrapper(self, payload: bytes) -> bytes:
        payload = payload.rstrip(b"\n")
        if not payload or b"\0" in payload:
            raise self.Error("FEL payload must be non-empty shell text")
        indented_payload = b"\n".join(b"    " + line for line in payload.splitlines())
        return self._RETRY_WRAPPER_PREFIX + indented_payload + b"\n" + self._RETRY_WRAPPER_SUFFIX

    def patch_ly_link_sh(self, content: bytes) -> bytes:
        if self._HOOK_MARKER in content:
            raise self.Error("lyLink.sh already contains the CatPlay FEL hook")
        shebang = b"#!/bin/sh\n"
        if not content.startswith(shebang):
            raise self.Error("lyLink.sh has an unexpected interpreter line")
        hook = self.build_retry_wrapper(self.build_payload())
        return shebang + b"\n" + hook + content[len(shebang) :].lstrip(b"\n")


class SquashfsPatcher:
    """Patch lyLink.sh inside the customer SquashFS image."""

    _SUPERBLOCK_SIZE = 96
    _MAGIC = b"hsqs"
    _XZ_COMPRESSION_ID = 4
    _BLOCK_SIZE = 262144
    _NO_XATTR_TABLE = 0xFFFFFFFFFFFFFFFF
    _METADATA_LINE = re.compile(r"^[bcdlps-][rwxstST-]{9}\s")
    _OWNER = re.compile(r"^[bcdlps-][rwxstST-]{9}\s+(\d+)/(\d+)\s")
    _CONSOLE = re.compile(
        r"^c[rwx-]{9}\s+0/0\s+5,\s*1\s+.*dev/console$", re.MULTILINE
    )

    class Error(RuntimeError):
        """Raised when the customer filesystem cannot be patched safely."""

    @dataclasses.dataclass(frozen=True)
    class Info:
        inodes: int
        mkfs_time: int
        block_size: int
        compression: int
        xattr_table: int
        bytes_used: int

    def __init__(self, script_patcher: BootScriptPatcher | None = None) -> None:
        self._script_patcher = script_patcher or BootScriptPatcher()
        self._unsquashfs = self._command("unsquashfs")
        self._mksquashfs = self._command("mksquashfs")

    def patch(self, content: bytes, *, size_limit: int, temp_parent: Path | None = None) -> bytes:
        original_info = self.inspect(content)
        if len(content) > size_limit:
            raise self.Error(
                f"customer image exceeds its partition: {len(content)} > {size_limit}"
            )
        if temp_parent is not None:
            temp_parent.mkdir(parents=True, exist_ok=True)
            if not temp_parent.is_dir():
                raise self.Error(f"temporary path is not a directory: {temp_parent}")

        with tempfile.TemporaryDirectory(prefix="cpbox2fel-", dir=temp_parent) as directory:
            workspace = Path(directory)
            original_path = workspace / "customer.original.sqfs"
            patched_path = workspace / "customer.patched.sqfs"
            root = workspace / "customer-root"
            original_path.write_bytes(content)

            listing = self._run(
                [self._unsquashfs, "-lln", "-full-precision", "-UTC", str(original_path)]
            ).decode(errors="replace")
            metadata = [
                line for line in listing.splitlines() if self._METADATA_LINE.match(line)
            ]
            if not metadata:
                raise self.Error("cannot parse the customer SquashFS listing")
            owners = [self._OWNER.match(line) for line in metadata]
            if any(match is None for match in owners):
                raise self.Error("cannot parse customer SquashFS ownership")
            if any(match.groups() != ("0", "0") for match in owners if match is not None):
                raise self.Error("customer contains non-root ownership; refusing to normalize it")
            special = [line for line in metadata if line[0] in "bcps"]
            if len(special) != 1 or self._CONSOLE.match(special[0]) is None:
                raise self.Error(
                    "customer special files are not exactly root-owned dev/console (5,1)"
                )

            self._run(
                [
                    self._unsquashfs,
                    "-no-xattrs",
                    "-ignore-errors",
                    "-no-exit-code",
                    "-d",
                    str(root),
                    str(original_path),
                ]
            )
            link_script = root / "app" / "lyLink.sh"
            if not link_script.is_file() or link_script.is_symlink():
                raise self.Error("customer has no regular app/lyLink.sh boot hook")
            if not link_script.resolve().is_relative_to(root.resolve()):
                raise self.Error("app/lyLink.sh escapes the extracted customer root")
            script_stat = link_script.stat()
            try:
                patched_script = self._script_patcher.patch_ly_link_sh(link_script.read_bytes())
            except BootScriptPatcher.Error as error:
                raise self.Error(str(error)) from error
            link_script.write_bytes(patched_script)
            os.chmod(link_script, script_stat.st_mode & 0o7777)
            os.utime(link_script, ns=(script_stat.st_atime_ns, script_stat.st_mtime_ns))

            dev_path = root / "dev"
            if not dev_path.is_dir():
                raise self.Error("extracted customer filesystem has no dev directory")
            console_time = int(dev_path.stat().st_mtime)
            self._run(
                [
                    self._mksquashfs,
                    str(root),
                    str(patched_path),
                    "-noappend",
                    "-comp",
                    "xz",
                    "-b",
                    str(self._BLOCK_SIZE),
                    "-all-root",
                    "-no-tailends",
                    "-exports",
                    "-no-xattrs",
                    "-mkfs-time",
                    str(original_info.mkfs_time),
                    "-processors",
                    "1",
                    "-no-progress",
                    "-p",
                    f"dev/console C {console_time} 600 0 0 5 1",
                ]
            )
            if not patched_path.is_file():
                raise self.Error("mksquashfs did not create the patched customer image")
            patched = patched_path.read_bytes()
            patched_info = self.inspect(patched)
            if len(patched) > size_limit:
                raise self.Error(
                    f"patched customer exceeds its partition: {len(patched)} > {size_limit}"
                )
            if patched_info.inodes != original_info.inodes:
                raise self.Error("rebuilt customer changed the inode count")

            extracted_script = self._run(
                [self._unsquashfs, "-cat", str(patched_path), "app/lyLink.sh"]
            )
            if extracted_script != patched_script:
                raise self.Error("rebuilt customer does not contain the exact patched script")
            console = self._run(
                [
                    self._unsquashfs,
                    "-lln",
                    "-full-precision",
                    "-UTC",
                    str(patched_path),
                    "dev/console",
                ]
            ).decode(errors="replace")
            if self._CONSOLE.search(console) is None:
                raise self.Error("rebuilt customer lost dev/console")
            return patched

    def inspect(self, content: bytes) -> Info:
        if len(content) < self._SUPERBLOCK_SIZE or content[:4] != self._MAGIC:
            raise self.Error("customer member is not little-endian SquashFS")
        info = self.Info(
            inodes=struct.unpack_from("<I", content, 4)[0],
            mkfs_time=struct.unpack_from("<I", content, 8)[0],
            block_size=struct.unpack_from("<I", content, 12)[0],
            compression=struct.unpack_from("<H", content, 20)[0],
            bytes_used=struct.unpack_from("<Q", content, 40)[0],
            xattr_table=struct.unpack_from("<Q", content, 56)[0],
        )
        if info.compression != self._XZ_COMPRESSION_ID or info.block_size != self._BLOCK_SIZE:
            raise self.Error(f"unsupported customer SquashFS geometry: {info}")
        if info.xattr_table != self._NO_XATTR_TABLE:
            raise self.Error("customer SquashFS has xattrs; refusing to discard them")
        if info.bytes_used > len(content):
            raise self.Error("customer SquashFS is truncated")
        return info

    def _command(self, name: str) -> str:
        path = shutil.which(name)
        if path is None:
            raise self.Error(
                f"required command is unavailable: {name}; install the squashfs-tools package"
            )
        return path

    def _run(self, command: list[str]) -> bytes:
        try:
            result = subprocess.run(
                command,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=False,
            )
        except OSError as error:
            raise self.Error(f"cannot execute {command[0]}: {error}") from error
        if result.returncode != 0:
            detail = (result.stderr or result.stdout).decode(errors="replace").strip()
            raise self.Error(
                f"command failed ({result.returncode}): {' '.join(command)}"
                f"{(': ' + detail) if detail else ''}"
            )
        return result.stdout


class Decrypter:
    """Encrypt and decrypt CPBox OTA data using a named device method."""

    _CPIO_CRC_MAGIC = b"070702"
    _CPIO_HEADER_SIZE = 110
    _CPIO_TRAILER_NAME = b"TRAILER!!!\0"
    _V821_XOR_KEY = bytes.fromhex(
        "1269544837f0d56e927e26c3e525c5ee2a968b51ab10f2a9dc1d1b54b2a0a4b5"
        "faf8fd31e9d2a07b50c63e36eb03246613bfb73fc0a9e89cc704f079a4952e9e"
        "7e3bd077fd70f24e3630842133a887c77742e40df6f32c895249915fbe9164e0"
        "af2d03c12bd809974759061c808fbd48b1a155c8958151e7cbe2478974ab6a13"
        "696dd40445de9b8d37a2a9b73167ff02375e0617e8eeb4aff2a42897395735c7"
        "62053e807530ceab6052cd94fa545b513361697b4f1d2a42c152d9fba90ec22c"
        "2401ac89317a3592cc0226c6568117e94ca0ba4346b40f2e925f1d080d750005"
        "620dc92322fdd4486c6f4f6fd52788a108a5ca9dd61b85a1fecc2b7277958550"
    )
    _KEY_BY_METHOD = {"v821": _V821_XOR_KEY}

    class Error(RuntimeError):
        """Raised when encryption parameters or OTA plaintext are invalid."""

    def __init__(self, method: str) -> None:
        try:
            self._key = self._KEY_BY_METHOD[method]
        except (KeyError, TypeError) as error:
            raise self.Error(f"unsupported encryption method: {method!r}") from error

    def decrypt(self, data: bytes) -> bytes:
        decrypted = self._xor(data)
        self._assert_cpio_trailer(decrypted)
        return decrypted

    def encrypt(self, data: bytes) -> bytes:
        self._assert_cpio_trailer(data)
        return self._xor(data)

    def _xor(self, data: bytes) -> bytes:
        return bytes(value ^ self._key[index & 0xFF] for index, value in enumerate(data))

    def _assert_cpio_trailer(self, data: bytes) -> None:
        name_offset = data.rfind(self._CPIO_TRAILER_NAME)
        header_offset = name_offset - self._CPIO_HEADER_SIZE
        if name_offset < 0 or header_offset < 0:
            raise self.Error("decrypted OTA does not end with a CPIO TRAILER!!! entry")

        header = data[header_offset:name_offset]
        if header[:6] != self._CPIO_CRC_MAGIC:
            raise self.Error("final CPIO entry is not in the SVR4 CRC format")
        try:
            fields = [int(header[6 + index * 8 : 14 + index * 8], 16) for index in range(13)]
        except ValueError as error:
            raise self.Error("final CPIO entry has an invalid header") from error

        file_size = fields[6]
        name_size = fields[11]
        if name_size != len(self._CPIO_TRAILER_NAME) or file_size != 0:
            raise self.Error(
                "final CPIO entry is not an empty TRAILER!!! "
                f"(name size={name_size:#x}, file size={file_size:#x})"
            )
        if any(data[name_offset + len(self._CPIO_TRAILER_NAME) :]):
            raise self.Error("non-zero data follows the final CPIO TRAILER!!! entry")


class SwuMetadataValidator:
    """Validate the device and partition layout declared by sw-description."""

    _PROJECT = re.compile(r'\bly_project_name\s*=\s*"ly([0-9]+)"\s*;')
    _MAGIC = re.compile(r'\bmagic\s*=\s*"0xAA55A55A"\s*;')
    _REQUIRED_LAYOUT = (
        'filename = "customer"',
        'device = "/dev/by-name/customer"',
        'filename = "kernel"',
        'device = "/dev/by-name/boot"',
    )

    class Error(RuntimeError):
        """Raised when sw-description does not match the expected OTA layout."""

    def validate(self, content: bytes, ly: str) -> None:
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError as error:
            raise self.Error("sw-description is not UTF-8") from error
        project = self._PROJECT.search(text)
        expected = ly.removeprefix("ly")
        if project is None or project.group(1) != expected:
            raise self.Error(f"sw-description does not target {ly}")
        if self._MAGIC.search(text) is None:
            raise self.Error("sw-description lacks the expected 0xAA55A55A magic")
        missing = [value for value in self._REQUIRED_LAYOUT if value not in text]
        if missing:
            raise self.Error(f"sw-description has an unexpected image layout; missing {missing}")


class SwuMetadataPatcher:
    """Re-arm the FEL hook in the final SWUpdate boot environment stage."""

    _STAGE_NAME = b"upgrade_usr"
    _COUNTER_NAME = b"catplay_fel_attempts"
    _COMPLETE_NAME = b"catplay_fel_complete"

    class Error(RuntimeError):
        """Raised when sw-description cannot be patched unambiguously."""

    def patch_fel_rearm(self, content: bytes) -> bytes:
        if not content:
            raise self.Error("sw-description is empty")
        if self._COUNTER_NAME in content or self._COMPLETE_NAME in content:
            raise self.Error("sw-description already contains CatPlay FEL environment entries")

        stage_start, stage_end = self._find_named_block(content, self._STAGE_NAME)
        stage = content[stage_start:stage_end]
        bootenv_match = re.search(rb"\bbootenv\s*:\s*\(", stage)
        if bootenv_match is None:
            raise self.Error("upgrade_usr has no bootenv section")
        open_offset = stage.find(b"(", bootenv_match.start())
        close_offset = self._find_matching(stage, open_offset, ord("("), ord(")"))
        body = stage[open_offset + 1 : close_offset]
        last_content = len(body.rstrip()) - 1
        if last_content < 0 or body[last_content] not in (ord("}"), ord(",")):
            raise self.Error("upgrade_usr bootenv has an unexpected entry layout")

        entry_indents = re.findall(rb"(?m)^([ \t]*)\{", body)
        if not entry_indents:
            raise self.Error("upgrade_usr bootenv contains no entries")
        entry_indent = entry_indents[-1]
        field_indent = entry_indent + b"    "
        separator = b"" if body[last_content] == ord(",") else b","
        injection = (
            separator
            + b"\n"
            + entry_indent
            + b"{\n"
            + field_indent
            + b'name = "'
            + self._COUNTER_NAME
            + b'";\n'
            + field_indent
            + b'value = "";\n'
            + entry_indent
            + b"},\n"
            + entry_indent
            + b"{\n"
            + field_indent
            + b'name = "'
            + self._COMPLETE_NAME
            + b'";\n'
            + field_indent
            + b'value = "";\n'
            + entry_indent
            + b"}"
        )
        insert_at = stage_start + open_offset + 1 + last_content + 1
        patched = content[:insert_at] + injection + content[insert_at:]
        self.validate_fel_rearm(patched)
        return patched

    def validate_fel_rearm(self, content: bytes) -> None:
        stage_start, stage_end = self._find_named_block(content, self._STAGE_NAME)
        stage = content[stage_start:stage_end]
        bootenv_match = re.search(rb"\bbootenv\s*:\s*\(", stage)
        if bootenv_match is None:
            raise self.Error("upgrade_usr has no bootenv section")
        open_offset = stage.find(b"(", bootenv_match.start())
        close_offset = self._find_matching(stage, open_offset, ord("("), ord(")"))
        bootenv = stage[open_offset + 1 : close_offset]
        for name in (self._COUNTER_NAME, self._COMPLETE_NAME):
            pattern = rb'\bname\s*=\s*"' + re.escape(name) + rb'"\s*;\s*value\s*=\s*""\s*;'
            if len(re.findall(pattern, bootenv, flags=re.DOTALL)) != 1:
                raise self.Error(
                    f"upgrade_usr bootenv does not clear {name.decode('ascii')} exactly once"
                )

    def _find_named_block(self, content: bytes, name: bytes) -> tuple[int, int]:
        matches = list(re.finditer(rb"\b" + re.escape(name) + rb"\s*=\s*\{", content))
        if len(matches) != 1:
            raise self.Error(
                f"expected exactly one {name.decode('ascii')} stage, found {len(matches)}"
            )
        open_offset = content.find(b"{", matches[0].start())
        close_offset = self._find_matching(content, open_offset, ord("{"), ord("}"))
        return open_offset + 1, close_offset

    def _find_matching(self, content: bytes, start: int, opening: int, closing: int) -> int:
        depth = 0
        index = start
        in_string = False
        in_comment = False
        while index < len(content):
            current = content[index]
            following = content[index + 1] if index + 1 < len(content) else None
            if in_comment:
                if current == ord("*") and following == ord("/"):
                    in_comment = False
                    index += 2
                    continue
            elif in_string:
                if current == ord("\\"):
                    index += 2
                    continue
                if current == ord('"'):
                    in_string = False
            elif current == ord("/") and following == ord("*"):
                in_comment = True
                index += 2
                continue
            elif current == ord('"'):
                in_string = True
            elif current == opening:
                depth += 1
            elif current == closing:
                depth -= 1
                if depth == 0:
                    return index
            index += 1
        raise self.Error("unterminated block in sw-description")


class OtaRepacker:
    """Patch the customer filesystem in an encrypted OTA file."""

    _MAX_INPUT_SIZE = 512 * 1024 * 1024
    _CUSTOMER_PARTITION_SIZE = {"ly5166": 0x3A0000, "ly5190": 0x3A0000}
    _REQUIRED_MEMBERS = {"sw-description", "customer", "kernel"}

    class Error(RuntimeError):
        """Raised when an OTA file cannot be validated or repacked."""

    @dataclasses.dataclass(frozen=True)
    class Result:
        input_path: Path
        output_path: Path
        input_size: int
        output_size: int

    def __init__(self, ly: str, method: str) -> None:
        try:
            self._customer_size_limit = self._CUSTOMER_PARTITION_SIZE[ly]
        except KeyError as error:
            raise self.Error(f"unsupported LY project for repacking: {ly!r}") from error
        self._ly = ly
        try:
            self._decrypter = Decrypter(method)
        except Decrypter.Error as error:
            raise self.Error(str(error)) from error
        self._envelope = UpdateImageEnvelope()
        self._metadata = SwuMetadataValidator()
        self._metadata_patcher = SwuMetadataPatcher()

    def repack(
        self,
        input_file: Path | str,
        output_file: Path | str,
        *,
        temp_parent: Path | None = None,
    ) -> Result:
        input_path = Path(input_file)
        output_path = Path(output_file)
        self._validate_paths(input_path, output_path)

        encrypted = input_path.read_bytes()
        if not encrypted:
            raise self.Error(f"input OTA is empty: {input_path}")
        if len(encrypted) > self._MAX_INPUT_SIZE:
            raise self.Error(
                f"input OTA exceeds the {self._MAX_INPUT_SIZE}-byte limit: {input_path}"
            )

        try:
            decrypted = self._decrypter.decrypt(encrypted)
            original_archive = self._envelope.unpack(decrypted)
            archive = CpioArchive.parse(original_archive)
            if archive.repack() != original_archive:
                raise self.Error("CPIO parser did not reproduce the input archive byte-for-byte")
            missing = self._REQUIRED_MEMBERS.difference(archive.names)
            if missing:
                raise self.Error(f"OTA lacks required CPIO members: {sorted(missing)}")
            self._metadata.validate(archive.member("sw-description"), self._ly)

            patched_metadata = self._metadata_patcher.patch_fel_rearm(
                archive.member("sw-description")
            )
            patched_customer = SquashfsPatcher().patch(
                archive.member("customer"),
                size_limit=self._customer_size_limit,
                temp_parent=temp_parent,
            )
            archive.replace_member("sw-description", patched_metadata)
            archive.replace_member("customer", patched_customer)
            patched_archive = archive.repack()
            reparsed = CpioArchive.parse(patched_archive)
            if reparsed.repack() != patched_archive:
                raise self.Error("patched CPIO failed round-trip validation")
            self._metadata.validate(reparsed.member("sw-description"), self._ly)
            self._metadata_patcher.validate_fel_rearm(
                reparsed.member("sw-description")
            )
            patched_decrypted = self._envelope.repack(patched_archive)
            patched_encrypted = self._decrypter.encrypt(patched_decrypted)
        except (
            CpioArchive.Error,
            Decrypter.Error,
            SquashfsPatcher.Error,
            SwuMetadataPatcher.Error,
            SwuMetadataValidator.Error,
            UpdateImageEnvelope.Error,
        ) as error:
            raise self.Error(str(error)) from error

        self._write_atomically(output_path, patched_encrypted)
        self._verify_output(output_path, patched_decrypted)
        return self.Result(
            input_path=input_path,
            output_path=output_path,
            input_size=len(encrypted),
            output_size=len(patched_encrypted),
        )

    def _validate_paths(self, input_path: Path, output_path: Path) -> None:
        if not input_path.exists():
            raise self.Error(f"input OTA does not exist: {input_path}")
        if not input_path.is_file() or input_path.is_symlink():
            raise self.Error(f"input OTA is not a regular file: {input_path}")
        if input_path.resolve() == output_path.resolve():
            raise self.Error("input and output OTA paths must be different")
        if output_path.exists():
            raise self.Error(f"output OTA already exists: {output_path}")
        output_path.parent.mkdir(parents=True, exist_ok=True)
        if not output_path.parent.is_dir():
            raise self.Error(f"output parent is not a directory: {output_path.parent}")

    def _write_atomically(self, target: Path, content: bytes) -> None:
        temporary_path: Path | None = None
        try:
            with tempfile.NamedTemporaryFile(
                mode="wb",
                dir=target.parent,
                prefix=f".{target.name}.",
                suffix=".part",
                delete=False,
            ) as output:
                temporary_path = Path(output.name)
                output.write(content)
                output.flush()
                os.fsync(output.fileno())
            temporary_path.replace(target)
        except OSError as error:
            raise self.Error(f"cannot save output OTA {target}: {error}") from error
        finally:
            if temporary_path is not None:
                temporary_path.unlink(missing_ok=True)

    def _verify_output(self, output_path: Path, expected_decrypted: bytes) -> None:
        if not output_path.is_file() or output_path.is_symlink():
            raise self.Error(f"output OTA is not a regular file: {output_path}")
        try:
            actual_decrypted = self._decrypter.decrypt(output_path.read_bytes())
            archive = self._envelope.unpack(actual_decrypted)
            CpioArchive.parse(archive)
        except (CpioArchive.Error, Decrypter.Error, UpdateImageEnvelope.Error) as error:
            raise self.Error(f"written OTA failed verification: {error}") from error
        if actual_decrypted != expected_decrypted:
            raise self.Error("written OTA differs from the repacked image")
