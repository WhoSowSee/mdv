import argparse
import hashlib
import io
from pathlib import Path
import re
import shlex
import shutil
import tarfile
import tomllib

from verify_deb import command, inspect_binary, verify_payload


TARGETS = {
    "x86_64-unknown-linux-gnu": "x86_64",
    "aarch64-unknown-linux-gnu": "aarch64",
}
RECIPES = Path(__file__).resolve().parents[1] / "packaging"


def context(package_format, target, binary):
    arch = TARGETS[target]
    if package_format == "arch" and arch != "x86_64":
        raise ValueError("Arch Linux packages require the x86_64 GNU target")
    metadata = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["package"]
    if binary is None:
        binary = Path("target") / target / "release" / metadata["name"]
    if binary.name != metadata["name"]:
        raise ValueError("The release binary name must match the package name")
    _, minimum, needs_libgcc = inspect_binary(binary, target)
    declared = metadata["metadata"]["deb"]["variants"]["glibc-2-39"]["depends"]
    baseline = re.fullmatch(r"libc6 \(>= (\d+(?:\.\d+)+)\), libgcc-s1", declared)
    if baseline is None or minimum is None:
        raise ValueError("Cannot determine the native GNU runtime baseline")
    baseline = baseline[1]
    if version_tuple(minimum) > version_tuple(baseline):
        raise ValueError(f"Binary requires glibc {minimum}, exceeding baseline {baseline}")
    return metadata, binary, arch, baseline, minimum, needs_libgcc


def version_tuple(version):
    return tuple(map(int, version.split(".")))


def sha256(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def prepare(package_format, target, directory, binary=None):
    metadata, binary, _, baseline, _, needs_libgcc = context(package_format, target, binary)
    directory.mkdir(parents=True, exist_ok=True)
    if any(directory.iterdir()):
        raise ValueError(f"Package staging directory must be empty: {directory}")
    fields = {
        key.upper(): metadata[key]
        for key in ("name", "version", "description", "license", "repository")
    }
    fields["GLIBC_MINIMUM"] = baseline
    if package_format == "rpm":
        sources = directory / "SOURCES"
        sources.mkdir()
        specs = directory / "SPECS"
        specs.mkdir()
        recipe = RECIPES / "mdv.spec.in"
        output = specs / f"{metadata['name']}.spec"
        fields = {key: value.replace("%", "%%") for key, value in fields.items()}
    else:
        sources = directory
        recipe = RECIPES / "PKGBUILD.in"
        output = directory / "PKGBUILD"
        fields = {key: shlex.quote(value) for key, value in fields.items()}
        dependencies = [f"glibc>={baseline}"] + (["libgcc"] if needs_libgcc else [])
        fields["DEPENDENCIES"] = " ".join(map(shlex.quote, dependencies))
        fields["BINARY_SHA256"] = sha256(binary)
        fields["LICENSE_SHA256"] = sha256(Path("LICENSE"))
    shutil.copyfile(binary, sources / metadata["name"])
    shutil.copyfile("LICENSE", sources / "LICENSE")
    template = recipe.read_text(encoding="utf-8")
    output.write_text(
        re.sub(r"@([A-Z_][A-Z_0-9]*)@", lambda match: fields[match[1]], template),
        encoding="utf-8",
        newline="\n",
    )
    print(f"Prepared {package_format} recipe: {output}")


def check_rpm_metadata(package, metadata, arch, baseline, minimum, needs_libgcc):
    command("rpmkeys", "--checksig", "--nosignature", str(package))
    fields = command(
        "rpm", "-qp", "--queryformat",
        "%{NAME}\n%{VERSION}\n%{RELEASE}\n%{ARCH}\n%{LICENSE}\n", str(package),
    ).splitlines()
    expected = [metadata["name"], metadata["version"], "1", arch, metadata["license"]]
    if fields != expected:
        raise ValueError(f"Unexpected RPM metadata: {fields!r}, expected {expected!r}")
    requires = set(command("rpm", "-qp", "--requires", str(package)).splitlines())
    required = {f"glibc >= {baseline}", "libc.so.6()(64bit)"}
    if needs_libgcc:
        required.add("libgcc_s.so.1()(64bit)")
    versions = re.findall(r"\(GLIBC_(\d+(?:\.\d+)+)\)\(64bit\)", "\n".join(requires))
    if (
        not required <= requires
        or not versions
        or max(map(version_tuple, versions)) < version_tuple(minimum)
    ):
        raise ValueError(
            f"RPM dependencies do not cover the release binary: {sorted(requires)}"
        )


def parse_pkginfo(text):
    fields = {}
    for line in text.splitlines():
        if line.startswith("#") or not line.strip():
            continue
        key, value = line.split(" = ", 1)
        fields.setdefault(key, []).append(value)
    return fields


def check_arch_metadata(text, metadata, arch, baseline, needs_libgcc):
    fields = parse_pkginfo(text)
    expected = {
        "pkgname": [metadata["name"]],
        "pkgver": [f"{metadata['version']}-1"],
        "arch": [arch],
        "license": [metadata["license"]],
    }
    if any(fields.get(key) != value for key, value in expected.items()):
        raise ValueError(f"Unexpected Arch package metadata: {fields!r}")
    required = {f"glibc>={baseline}"} | ({"libgcc"} if needs_libgcc else set())
    if set(fields.get("depend", [])) != required:
        raise ValueError(f"Unexpected Arch runtime dependencies: {fields.get('depend')!r}")


def check_payload(payload, binary, license_file, package_format):
    verify_payload(binary, payload)
    license_path = f"usr/share/licenses/{binary.name}/LICENSE"
    allowed = {f"usr/bin/{binary.name}", license_path}
    if package_format == "arch":
        allowed |= {".PKGINFO", ".BUILDINFO", ".MTREE"}
    with tarfile.open(fileobj=io.BytesIO(payload), mode="r:") as archive:
        entries = [
            entry for entry in archive
            if entry.name.removeprefix("./") == license_path
        ]
        if len(entries) != 1 or not entries[0].isfile() or entries[0].mode != 0o644:
            raise ValueError("Expected one regular license file with mode 0644")
        with archive.extractfile(entries[0]) as packaged:
            if packaged.read() != license_file.read_bytes():
                raise ValueError("Packaged license differs from LICENSE")
        for entry in archive:
            if not entry.isdir() and (
                not entry.isfile() or entry.name.removeprefix("./") not in allowed
            ):
                raise ValueError(f"Unexpected package payload entry: {entry.name}")


def verify(package_format, target, package, binary=None):
    metadata, binary, arch, baseline, minimum, needs_libgcc = context(package_format, target, binary)
    package = package.resolve()
    if package_format == "rpm":
        check_rpm_metadata(package, metadata, arch, baseline, minimum, needs_libgcc)
        payload = command("rpm2archive", "--nocompression", str(package), text=False)
    else:
        payload = command(
            "bsdtar", "--format", "pax", "-cf", "-", f"@{package}", text=False
        )
        with tarfile.open(fileobj=io.BytesIO(payload), mode="r:") as archive:
            info = [
                entry for entry in archive
                if entry.name.removeprefix("./") == ".PKGINFO"
            ]
            if len(info) != 1 or not info[0].isfile():
                raise ValueError("Expected one regular .PKGINFO file")
            with archive.extractfile(info[0]) as packaged:
                check_arch_metadata(
                    packaged.read().decode("utf-8"), metadata, arch, baseline,
                    needs_libgcc,
                )
    check_payload(payload, binary, Path("LICENSE"), package_format)
    print(f"Verified {package.name}: architecture={arch}, glibc>={baseline}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Prepare and verify native RPM and Arch packages.")
    parser.add_argument("action", choices=("prepare", "verify"))
    parser.add_argument("package_format", choices=("rpm", "arch"))
    parser.add_argument("target", choices=TARGETS)
    parser.add_argument("path", type=Path)
    parser.add_argument("--binary", type=Path)
    args = parser.parse_args()
    operation = prepare if args.action == "prepare" else verify
    operation(args.package_format, args.target, args.path, args.binary)
