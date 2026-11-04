import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import unittest
import zipfile


ROOT = Path(__file__).resolve().parents[2]
PUBLIC = ROOT / "website" / "public"
SH = shutil.which("sh")
POWERSHELL = shutil.which("pwsh") or shutil.which("powershell")
API = "https://api.github.com/repos/WhoSowSee/mdv/releases"


def shell_path(path):
    value = Path(path).resolve().as_posix()
    if os.name == "nt":
        return "/" + value[0].lower() + value[2:]
    return value


class InstallerFixtures(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="mdv-installer-tests-")
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name)
        self.destination = self.work / "install with spaces"
        self.scratch = self.work / "scratch"
        self.scratch.mkdir()
        self.archive = self.work / "release.zip"
        self.release = self.work / "release.json"
        self.log = self.work / "requests.log"
        self.target = "x86_64-unknown-linux-musl"
        self.tag = "v6.1.0"
        self.binary_name = "mdv"

    def fixture(self, payload=b"new mdv", digest=None, entries=None, **metadata):
        with zipfile.ZipFile(self.archive, "w") as archive:
            archive.writestr(self.binary_name, payload)
            for name, content in entries or []:
                archive.writestr(name, content)
        asset = {
            "name": f"mdv-{self.tag}-{self.target}.zip",
            "digest": digest or "sha256:" + hashlib.sha256(self.archive.read_bytes()).hexdigest(),
            "uploader": {"name": "nested name", "digest": "nested digest"},
        }
        release = {
            "body": 'Example: "name": "not an asset" and escaped \\ content',
            "assets": [asset],
            "tag_name": self.tag,
        }
        release.update(metadata)
        self.release.write_text(json.dumps(release, indent=2), encoding="utf-8")
        return release

    def installed(self, payload=b"old mdv"):
        self.destination.mkdir(exist_ok=True)
        binary = self.destination / self.binary_name
        binary.write_bytes(payload)
        return binary

    def assert_cleaned(self):
        self.assertEqual(list(self.scratch.iterdir()), [])
        if self.destination.exists():
            self.assertFalse(list(self.destination.glob(".mdv*")))


@unittest.skipUnless(SH and shutil.which("unzip"), "Requires sh and unzip")
class ShellInstallerTests(InstallerFixtures):
    def setUp(self):
        super().setUp()
        self.commands = self.work / "commands"
        self.commands.mkdir()
        self.write_command("uname", 'case "$1" in -s) echo "$MDV_TEST_OS";; -m) echo "$MDV_TEST_ARCH";; esac\n')
        self.write_command("shasum", '[ "$1" = "-a" ] && [ "$2" = "256" ]; sha256sum "$3"\n')
        self.write_command("curl", '''
while [ "$#" -gt 0 ]; do
    case "$1" in --output) output=$2; shift 2;; *) url=$1; shift;; esac
done
printf '%s\\n' "$url" >> "$MDV_TEST_ROOT/requests.log"
case "$url" in
    https://api.github.com/repos/WhoSowSee/mdv/releases/*) cp "$MDV_TEST_ROOT/release.json" "$output";;
    https://github.com/WhoSowSee/mdv/releases/download/*) cp "$MDV_TEST_ROOT/release.zip" "$output";;
    *) exit 40;;
esac
''')
        self.env = dict(os.environ, PATH=str(self.commands) + os.pathsep + os.environ["PATH"])
        self.env.update({
            "MDV_TEST_ROOT": shell_path(self.work),
            "MDV_TEST_COMMANDS": shell_path(self.commands),
            "MDV_TEST_OS": "Linux", "MDV_TEST_ARCH": "x86_64",
            "TMPDIR": shell_path(self.scratch), "TERMUX_VERSION": "", "MDV_INSTALL_DIR": "",
        })

    def write_command(self, name, body):
        path = self.commands / name
        path.write_text("#!/bin/sh\nset -eu\n" + body, encoding="utf-8", newline="\n")
        path.chmod(0o755)

    def run_installer(self, *extra, piped=False):
        arguments = [shell_path(PUBLIC / "install.sh")]
        if piped:
            arguments = ["-s", "--"]
        arguments += ["--install-dir", shell_path(self.destination), *extra]
        command = [SH, "-c", 'PATH="$MDV_TEST_COMMANDS:$PATH"; export PATH; exec sh "$@"',
                   "installer-test", *arguments]
        return subprocess.run(command, input=(PUBLIC / "install.sh").read_text() if piped else None,
                              env=self.env, capture_output=True, text=True, timeout=30)

    def test_install_and_update_through_pipe(self):
        self.fixture(entries=[("../outside", b"must not extract")])
        first = self.run_installer(piped=True)
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual((self.destination / "mdv").read_bytes(), b"new mdv")
        self.assertFalse((self.work / "outside").exists())
        self.assertEqual(self.log.read_text().splitlines()[0], API + "/latest")
        self.fixture(payload=b"updated mdv")
        update = self.run_installer()
        self.assertEqual(update.returncode, 0, update.stderr)
        self.assertEqual((self.destination / "mdv").read_bytes(), b"updated mdv")
        self.assert_cleaned()

    def test_platform_mapping_and_pinned_release(self):
        rows = [("Linux", "x86_64", "x86_64-unknown-linux-musl"),
                ("Linux", "aarch64", "aarch64-unknown-linux-musl"),
                ("Darwin", "x86_64", "x86_64-apple-darwin"),
                ("Darwin", "arm64", "aarch64-apple-darwin")]
        for system, architecture, target in rows:
            with self.subTest(system=system, architecture=architecture):
                self.env.update(MDV_TEST_OS=system, MDV_TEST_ARCH=architecture)
                self.target = target
                self.fixture()
                result = self.run_installer("--version", self.tag)
                self.assertEqual(result.returncode, 0, result.stderr)
                requests = self.log.read_text().splitlines()
                self.assertEqual(requests[-2], API + "/tags/" + self.tag)
                self.assertTrue(requests[-1].endswith(f"mdv-{self.tag}-{target}.zip"))
                self.assert_cleaned()

    def test_checksum_mismatch_preserves_installation(self):
        binary = self.installed()
        self.fixture(digest="sha256:" + "0" * 64)
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 mismatch", result.stderr)
        self.assertEqual(binary.read_bytes(), b"old mdv")
        self.assert_cleaned()

    def test_missing_digest_and_duplicate_asset_stop_before_download(self):
        for case in ("missing", "duplicate"):
            with self.subTest(case=case):
                binary = self.installed()
                release = self.fixture()
                if case == "missing": release["assets"][0]["digest"] = None
                else: release["assets"] *= 2
                self.release.write_text(json.dumps(release), encoding="utf-8")
                self.log.write_text("")
                result = self.run_installer()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(len(self.log.read_text().splitlines()), 1)
                self.assertEqual(binary.read_bytes(), b"old mdv")
                self.assert_cleaned()

    def test_bad_archive_preserves_installation(self):
        binary = self.installed()
        self.fixture(payload=b"")
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Release binary is empty", result.stderr)
        self.assertEqual(binary.read_bytes(), b"old mdv")
        self.assert_cleaned()

    def test_unsupported_platform_and_invalid_tag_do_not_download(self):
        self.env["MDV_TEST_OS"] = "FreeBSD"
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Unsupported platform", result.stderr)
        self.assertFalse(self.log.exists())
        self.env["MDV_TEST_OS"] = "Linux"
        result = self.run_installer("--version", "../../main")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("Invalid release tag", result.stderr)
        self.assertFalse(self.log.exists())


@unittest.skipUnless(os.name == "nt" and POWERSHELL, "Requires Windows PowerShell")
class PowerShellInstallerTests(InstallerFixtures):
    def setUp(self):
        super().setUp()
        self.binary_name = "mdv.exe"
        self.target = "aarch64-pc-windows-msvc" if platform.machine().lower() in ("arm64", "aarch64") else "x86_64-pc-windows-msvc"
        self.runner = self.work / "run-test.ps1"
        self.runner.write_text('''
param($Installer, $FixtureRoot, $Destination, $Version)
$ErrorActionPreference = 'Stop'
function Invoke-RestMethod {
    param($Uri, $Headers, [switch]$UseBasicParsing)
    Add-Content -LiteralPath (Join-Path $FixtureRoot 'requests.log') -Value $Uri
    Get-Content -LiteralPath (Join-Path $FixtureRoot 'release.json') -Raw | ConvertFrom-Json
}
function Invoke-WebRequest {
    param($Uri, $OutFile, $Headers, [switch]$UseBasicParsing)
    Add-Content -LiteralPath (Join-Path $FixtureRoot 'requests.log') -Value $Uri
    Copy-Item -LiteralPath (Join-Path $FixtureRoot 'release.zip') -Destination $OutFile
}
$before = [Environment]::GetEnvironmentVariable('Path', 'User')
try {
    $source = Get-Content -LiteralPath $Installer -Raw
    & ([scriptblock]::Create($source)) -InstallDir $Destination -Version $Version -NoModifyPath
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
if ([Environment]::GetEnvironmentVariable('Path', 'User') -cne $before) { throw 'User PATH changed' }
''', encoding="utf-8")

    def run_installer(self, version="latest", executable=POWERSHELL):
        env = dict(os.environ, TMP=str(self.scratch), TEMP=str(self.scratch))
        env.pop("PSModulePath", None)
        return subprocess.run([executable, "-NoProfile", "-NonInteractive", "-File", str(self.runner),
                               str(PUBLIC / "install.ps1"), str(self.work), str(self.destination), version],
                              env=env, capture_output=True, text=True, timeout=30)

    def test_install_update_and_pinned_release(self):
        self.fixture(entries=[("../outside", b"must not extract")])
        result = self.run_installer()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.destination / "mdv.exe").read_bytes(), b"new mdv")
        self.assertFalse((self.work / "outside").exists())
        self.fixture(payload=b"updated mdv")
        result = self.run_installer(self.tag)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.destination / "mdv.exe").read_bytes(), b"updated mdv")
        self.assertIn(API + "/tags/" + self.tag, self.log.read_text())
        self.assert_cleaned()

    def test_windows_powershell_51(self):
        legacy = shutil.which("powershell")
        if not legacy: self.skipTest("Windows PowerShell 5.1 not available")
        self.fixture()
        result = self.run_installer(executable=legacy)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.destination / "mdv.exe").read_bytes(), b"new mdv")
        self.fixture(payload=b"updated mdv")
        result = self.run_installer(executable=legacy)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.destination / "mdv.exe").read_bytes(), b"updated mdv")
        self.assert_cleaned()

    def test_checksum_mismatch_preserves_installation(self):
        binary = self.installed()
        self.fixture(digest="sha256:" + "0" * 64)
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("SHA-256 mismatch", result.stderr)
        self.assertEqual(binary.read_bytes(), b"old mdv")
        self.assert_cleaned()

    def test_missing_digest_and_duplicate_asset(self):
        for case in ("missing", "duplicate"):
            with self.subTest(case=case):
                binary = self.installed()
                release = self.fixture()
                if case == "missing": release["assets"][0].pop("digest")
                else: release["assets"] *= 2
                self.release.write_text(json.dumps(release), encoding="utf-8")
                result = self.run_installer()
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(binary.read_bytes(), b"old mdv")
                self.assert_cleaned()

    def test_empty_binary_is_rejected(self):
        binary = self.installed()
        self.fixture(payload=b"")
        result = self.run_installer()
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(binary.read_bytes(), b"old mdv")
        self.assert_cleaned()


if __name__ == "__main__":
    unittest.main()
