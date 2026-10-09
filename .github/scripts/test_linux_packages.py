import io
import os
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch

import linux_packages as packages


METADATA = {
    "name": "mdv",
    "version": "1.2.3",
    "license": "MIT",
}
PKGINFO = """pkgname = mdv
pkgver = 1.2.3-1
arch = x86_64
license = MIT
depend = glibc>=2.39
depend = libgcc
"""
REQUIRES = """glibc >= 2.39
libc.so.6()(64bit)
libc.so.6(GLIBC_2.34)(64bit)
libgcc_s.so.1()(64bit)
"""


class LinuxPackageTests(unittest.TestCase):
    def setUp(self):
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        self.root = Path(directory.name)
        previous = Path.cwd()
        os.chdir(self.root)
        self.addCleanup(os.chdir, previous)
        Path("Cargo.toml").write_text(
            """[package]
name = "mdv"
version = "1.2.3"
description = "Terminal Markdown Viewer"
license = "MIT"
repository = "https://github.com/WhoSowSee/mdv"
[package.metadata.deb.variants.glibc-2-39]
depends = "libc6 (>= 2.39), libgcc-s1"
""",
            encoding="utf-8",
        )
        self.binary = self.root / "mdv"
        self.binary.write_bytes(b"original release binary")
        self.license = self.root / "LICENSE"
        self.license.write_text("MIT License\n", encoding="utf-8", newline="\n")

    def prepare(self, package_format, target="x86_64-unknown-linux-gnu"):
        with patch.object(packages, "inspect_binary", return_value=("amd64", "2.34", True)):
            packages.prepare(package_format, target, self.root / "staging", self.binary)

    def test_arch_recipe_uses_manifest_version_and_verified_source_checksums(self):
        self.prepare("arch")
        recipe = (self.root / "staging/PKGBUILD").read_text(encoding="utf-8")
        self.assertIn("pkgver=1.2.3", recipe)
        self.assertIn(packages.sha256(self.binary), recipe)
        self.assertIn(packages.sha256(self.license), recipe)
        self.assertNotIn("SKIP", recipe)
        self.assertEqual((self.root / "staging/mdv").read_bytes(), self.binary.read_bytes())

    def test_rpm_recipe_stages_the_same_binary_and_license_for_both_native_targets(self):
        for target in packages.TARGETS:
            with self.subTest(target=target):
                staging = self.root / target
                with patch.object(packages, "inspect_binary", return_value=("amd64", "2.34", True)):
                    packages.prepare("rpm", target, staging, self.binary)
                self.assertIn("Version: 1.2.3", (staging / "SPECS/mdv.spec").read_text())
                self.assertEqual((staging / "SOURCES/mdv").read_bytes(), self.binary.read_bytes())
                self.assertEqual((staging / "SOURCES/LICENSE").read_bytes(), self.license.read_bytes())

    def test_stale_staging_directory_is_rejected_without_overwriting_files(self):
        staging = self.root / "staging"
        staging.mkdir()
        owned = staging / "owned"
        owned.write_text("keep", encoding="utf-8")
        with self.assertRaisesRegex(ValueError, "must be empty"):
            self.prepare("rpm")
        self.assertEqual(owned.read_text(), "keep")

    def test_arch_arm_and_glibc_above_the_manifest_baseline_are_rejected(self):
        with self.assertRaisesRegex(ValueError, "x86_64"):
            packages.context("arch", "aarch64-unknown-linux-gnu", self.binary)
        with (
            patch.object(packages, "inspect_binary", return_value=("amd64", "2.40", True)),
            self.assertRaisesRegex(ValueError, "exceeding baseline 2.39"),
        ):
            packages.context("rpm", "x86_64-unknown-linux-gnu", self.binary)

    def rpm_metadata(self, fields="mdv\n1.2.3\n1\nx86_64\nMIT\n", requires=REQUIRES):
        with patch.object(packages, "command", side_effect=["digests OK", fields, requires]):
            packages.check_rpm_metadata(Path("mdv.rpm"), METADATA, "x86_64", "2.39", "2.34", True)

    def test_rpm_metadata_rejects_stale_versions_and_wrong_architecture(self):
        self.rpm_metadata()
        for fields in ["mdv\n1.2.2\n1\nx86_64\nMIT\n", "mdv\n1.2.3\n1\naarch64\nMIT\n"]:
            with self.subTest(fields=fields), self.assertRaisesRegex(ValueError, "metadata"):
                self.rpm_metadata(fields)

    def test_rpm_dependencies_require_sonames_and_adequate_symbol_versions(self):
        for requires in [
            REQUIRES.replace("libgcc_s.so.1()(64bit)\n", ""),
            REQUIRES.replace("libc.so.6()(64bit)\n", ""),
            REQUIRES.replace("GLIBC_2.34", "GLIBC_2.17"),
            REQUIRES.replace("glibc >= 2.39\n", ""),
        ]:
            with self.subTest(requires=requires), self.assertRaisesRegex(ValueError, "dependencies"):
                self.rpm_metadata(requires=requires)

    def test_arch_metadata_rejects_wrong_versions_architecture_and_dependencies(self):
        packages.check_arch_metadata(PKGINFO, METADATA, "x86_64", "2.39", True)
        for text in [
            PKGINFO.replace("1.2.3-1", "1.2.2-1"),
            PKGINFO.replace("x86_64", "aarch64"),
            PKGINFO.replace("glibc>=2.39", "glibc"),
            PKGINFO.replace("depend = libgcc\n", ""),
            PKGINFO + "arch = aarch64\n",
        ]:
            with self.subTest(text=text), self.assertRaisesRegex(ValueError, "metadata|dependencies"):
                packages.check_arch_metadata(text, METADATA, "x86_64", "2.39", True)

    def payload(self, license_content=b"MIT License\n", license_mode=0o644, extra=None):
        stream = io.BytesIO()
        files = [
            ("usr/bin/mdv", self.binary.read_bytes(), 0o755),
            ("usr/share/licenses/mdv/LICENSE", license_content, license_mode),
        ]
        if extra is not None:
            files.append(extra)
        with tarfile.open(fileobj=stream, mode="w") as archive:
            for name, content, mode in files:
                entry = tarfile.TarInfo(name)
                entry.mode = mode
                entry.size = len(content)
                archive.addfile(entry, io.BytesIO(content))
        return stream.getvalue()

    def test_payload_rejects_changed_license_wrong_permissions_and_extra_files(self):
        packages.check_payload(self.payload(), self.binary, self.license, "rpm")
        for payload in [
            self.payload(license_content=b"wrong license"),
            self.payload(license_mode=0o600),
            self.payload(extra=("usr/bin/other", b"unexpected", 0o755)),
        ]:
            with self.subTest(payload=payload), self.assertRaises(ValueError):
                packages.check_payload(payload, self.binary, self.license, "rpm")


if __name__ == "__main__":
    unittest.main()
