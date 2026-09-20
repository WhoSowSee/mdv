import io
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import verify_deb


class DebianValidationTests(unittest.TestCase):
    def write_binary(self, content):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.binary = Path(directory.name) / "mdv"
        self.binary.write_bytes(content)

    def elf(self, machine=62, endian="little"):
        header = b"\x7fELF" + bytes([2, 1 if endian == "little" else 2]) + bytes(12)
        self.write_binary(header + machine.to_bytes(2, endian))

    def resolve(self, dynamic, versions, target="x86_64-unknown-linux-gnu"):
        with patch.object(verify_deb, "command", side_effect=[dynamic, versions]):
            return verify_deb.inspect_binary(self.binary, target)

    def payload(self, content, mode=0o755):
        buffer = io.BytesIO()
        with tarfile.open(fileobj=buffer, mode="w") as archive:
            entry = tarfile.TarInfo("usr/bin/mdv")
            entry.mode = mode
            entry.size = len(content)
            archive.addfile(entry, io.BytesIO(content))
        return buffer.getvalue()

    def test_gnu_inspection_uses_target_architecture_and_numeric_symbol_versions(self):
        self.elf()
        dynamic = "(NEEDED) Shared library: [libc.so.6]\n(NEEDED) Shared library: [libgcc_s.so.1]"
        versions = "Name: GLIBC_2.9\nName: GLIBC_2.34\nName: GLIBC_2.17"
        self.assertEqual(self.resolve(dynamic, versions), ("amd64", "2.34", True))

    def test_sparc_inspection_reads_big_endian_machine(self):
        self.elf(machine=43, endian="big")
        self.assertEqual(
            self.resolve(
                "(NEEDED) Shared library: [libc.so.6]",
                "Name: GLIBC_2.18",
                "sparc64-unknown-linux-gnu",
            ),
            ("sparc64", "2.18", False),
        )

    def test_static_musl_has_no_system_dependencies(self):
        self.elf()
        self.assertEqual(
            self.resolve("No dynamic section", "LOAD", "x86_64-unknown-linux-musl"),
            ("amd64", None, False),
        )

    def test_unrecognised_libraries_and_glibc_abi_are_rejected(self):
        self.elf()
        cases = [
            ("(NEEDED) Shared library: [libssl.so.3]", "Name: GLIBC_2.34"),
            ("(NEEDED) Shared library: [libc.so.6]", "Name: GLIBC_PRIVATE"),
            ("(NEEDED) Shared library: [libc.so.6]", "Name: GLIBC_ABI_DT_RELR"),
            ("(NEEDED) Shared library: [libc.so.6]", ""),
        ]
        for dynamic, versions in cases:
            with (
                self.subTest(dynamic=dynamic, versions=versions),
                self.assertRaises(ValueError),
            ):
                self.resolve(dynamic, versions)

    def test_musl_with_a_loader_is_rejected(self):
        self.elf()
        with self.assertRaises(ValueError):
            self.resolve("", "INTERP", "x86_64-unknown-linux-musl")

    def test_wrong_architecture_is_rejected(self):
        self.elf(machine=183)
        with self.assertRaises(ValueError):
            verify_deb.inspect_binary(self.binary, "x86_64-unknown-linux-gnu")

    def test_declared_glibc_must_cover_binary_requirements(self):
        verify_deb.check_dependencies("libc6 (>= 2.39), libgcc-s1", "2.39", True)
        verify_deb.check_dependencies("libgcc-s1, libc6 (>= 2.39)", "2.34", True)
        with self.assertRaisesRegex(ValueError, "Binary requires glibc 2.40"):
            verify_deb.check_dependencies("libc6 (>= 2.39), libgcc-s1", "2.40", True)

    def test_missing_or_host_architecture_dependencies_are_rejected(self):
        for declared in ("", "libc6 (>= 2.39)", "libc6-i386 (>= 2.34), lib32gcc-s1"):
            with self.subTest(declared=declared), self.assertRaises(ValueError):
                verify_deb.check_dependencies(declared, "2.18", True)

    def test_static_package_must_not_declare_glibc(self):
        verify_deb.check_dependencies("", None, False)
        with self.assertRaises(ValueError):
            verify_deb.check_dependencies("libc6 (>= 2.39)", None, False)

    def test_payload_requires_public_executable_permissions(self):
        self.write_binary(b"release binary")
        verify_deb.verify_payload(self.binary, self.payload(b"release binary"))
        for mode in (0o700, 0o644, 0o4755):
            with (
                self.subTest(mode=oct(mode)),
                self.assertRaisesRegex(ValueError, "mode 0755"),
            ):
                verify_deb.verify_payload(
                    self.binary, self.payload(b"release binary", mode)
                )

    def test_payload_must_match_release_binary(self):
        self.write_binary(b"release binary")
        with self.assertRaisesRegex(ValueError, "differs from the release binary"):
            verify_deb.verify_payload(self.binary, self.payload(b"stale binary"))


if __name__ == "__main__":
    unittest.main()
