import argparse
import hashlib
import io
import os
from pathlib import Path
import re
import struct
import subprocess
import tarfile
import tomllib


TARGETS = {
    "aarch64-unknown-linux-gnu": ("arm64", 2, 1, 183),
    "aarch64-unknown-linux-musl": ("arm64", 2, 1, 183),
    "x86_64-unknown-linux-gnu": ("amd64", 2, 1, 62),
    "x86_64-unknown-linux-musl": ("amd64", 2, 1, 62),
    "i686-unknown-linux-gnu": ("i386", 1, 1, 3),
    "riscv64gc-unknown-linux-gnu": ("riscv64", 2, 1, 243),
    "sparc64-unknown-linux-gnu": ("sparc64", 2, 2, 43),
}
GLIBC_LIBRARIES = {
    "libc.so.6",
    "libm.so.6",
    "libpthread.so.0",
    "libdl.so.2",
    "librt.so.1",
    "libutil.so.1",
    "ld-linux-x86-64.so.2",
    "ld-linux-aarch64.so.1",
    "ld-linux.so.2",
    "ld-linux-riscv64-lp64d.so.1",
    "ld64.so.1",
}


def command(*args, text=True):
    return subprocess.check_output(args, text=text, env={**os.environ, "LC_ALL": "C"})


def inspect_binary(binary, target):
    with binary.open("rb") as stream:
        header = stream.read(20)
    arch, elf_class, endian, machine = TARGETS[target]
    if len(header) < 20 or header[:6] != b"\x7fELF" + bytes([elf_class, endian]):
        raise ValueError(f"Unexpected ELF format for {target}: {binary}")
    if struct.unpack("<H" if endian == 1 else ">H", header[18:20])[0] != machine:
        raise ValueError(f"Unexpected ELF machine for {target}: {binary}")

    dynamic = command("readelf", "--wide", "--dynamic", str(binary))
    needed = set(re.findall(r"\(NEEDED\).*?\[(.*?)\]", dynamic))
    if target.endswith("-musl"):
        program_headers = command("readelf", "--wide", "--program-headers", str(binary))
        if needed or re.search(r"\bINTERP\b", program_headers):
            raise ValueError(f"The musl binary must be static, found: {sorted(needed)}")
        return arch, None, False

    unknown = needed - GLIBC_LIBRARIES - {"libgcc_s.so.1"}
    if unknown or "libc.so.6" not in needed:
        raise ValueError(
            f"Unsupported runtime libraries for {target}: {sorted(needed)}"
        )
    versions = command("readelf", "--wide", "--version-info", str(binary))
    glibc = re.findall(r"Name: GLIBC_(\S+)", versions)
    if not glibc or any(
        not re.fullmatch(r"\d+(?:\.\d+)+", version) for version in glibc
    ):
        raise ValueError(f"Cannot determine a public glibc requirement for {target}")
    minimum = max(glibc, key=lambda version: tuple(map(int, version.split("."))))
    return arch, minimum, "libgcc_s.so.1" in needed


def check_dependencies(declared, minimum_glibc, needs_libgcc):
    parts = {part.strip() for part in declared.split(",") if part.strip()}
    if minimum_glibc is None:
        if parts:
            raise ValueError(
                f"Static musl package has runtime dependencies: {declared!r}"
            )
        return

    glibc = [part for part in parts if part.startswith("libc6 ")]
    match = (
        re.fullmatch(r"libc6 \(>= (\d+(?:\.\d+)+)\)", glibc[0])
        if len(glibc) == 1
        else None
    )
    expected = set(glibc) | ({"libgcc-s1"} if needs_libgcc else set())
    if match is None or parts != expected:
        raise ValueError(f"Unexpected Debian runtime dependencies: {declared!r}")
    declared_version = tuple(map(int, match[1].split(".")))
    required_version = tuple(map(int, minimum_glibc.split(".")))
    if declared_version < required_version:
        raise ValueError(
            f"Binary requires glibc {minimum_glibc}, but package declares {match[1]}; "
            "update the Cargo.toml Debian variant or the build environment"
        )


def verify_payload(binary, payload):
    expected_path = f"usr/bin/{binary.name}"
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:") as archive:
        entries = [
            entry for entry in archive if entry.name.removeprefix("./") == expected_path
        ]
        if len(entries) != 1 or not entries[0].isfile():
            raise ValueError(f"Expected one regular file at {expected_path}")
        entry = entries[0]
        if entry.mode != 0o755:
            raise ValueError(
                f"Expected mode 0755 for {expected_path}, got {entry.mode:04o}"
            )
        with archive.extractfile(entry) as packaged, binary.open("rb") as source:
            if (
                hashlib.file_digest(packaged, "sha256").digest()
                != hashlib.file_digest(source, "sha256").digest()
            ):
                raise ValueError("Packaged binary differs from the release binary")


def verify(target, package):
    metadata = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["package"]
    name = metadata["name"]
    binary = Path("target") / target / "release" / name
    arch, minimum_glibc, needs_libgcc = inspect_binary(binary, target)
    package = package.resolve()
    for field, expected in (("Package", name), ("Architecture", arch)):
        actual = command("dpkg-deb", "--field", str(package), field).strip()
        if actual != expected:
            raise ValueError(
                f"{package.name}: {field}={actual!r}, expected {expected!r}"
            )
    declared = command("dpkg-deb", "--field", str(package), "Depends").strip()
    check_dependencies(declared, minimum_glibc, needs_libgcc)
    payload = command("dpkg-deb", "--fsys-tarfile", str(package), text=False)
    verify_payload(binary, payload)
    print(f"Verified {package.name}: architecture={arch}, depends={declared!r}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("target", choices=TARGETS)
    parser.add_argument("package", type=Path)
    args = parser.parse_args()
    verify(args.target, args.package)
