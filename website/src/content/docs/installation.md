---
title: "Installation"
description: "Install, update or build mdv on Windows, Linux and macOS."
slug: "installation"
group: "Getting started"
order: 1
---

## Install in one command

The installers download the latest GitHub release for your operating system and architecture, verify its SHA-256 digest and install the executable. Rust is not required. Run the same command again to update.

### Windows

Run in Windows PowerShell 5.1 or PowerShell 7:

```powershell
irm https://whosowsee.github.io/mdv/install.ps1 | iex
```

Supports x64 and ARM64. Installs to `%LOCALAPPDATA%\Programs\mdv` and adds this directory to the user `PATH` and the current PowerShell session. Administrator rights are not required.

### Linux and macOS

```sh
curl -fsSL https://whosowsee.github.io/mdv/install.sh | sh
```

Supports x86_64 and ARM64 on Linux, and Intel and Apple Silicon on macOS. Requires `curl`, `unzip` and the system SHA-256 utility (`sha256sum` on Linux, `shasum` on macOS). Linux uses the static musl build. Installs to `~/.local/bin` without modifying shell profiles.

If this directory is not on your `PATH`, add the following line to your shell profile and run it in the current shell:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

### Select a version or directory

Pass the release tag directly when running the installer. The installation directory is optional:

Linux and macOS:

```sh
curl -fsSL https://whosowsee.github.io/mdv/install.sh | sh -s -- --version 'v<VERSION>'
```

Windows:

```powershell
& ([scriptblock]::Create((irm https://whosowsee.github.io/mdv/install.ps1))) -Version 'v<VERSION>'
```

Replace `v<VERSION>` with an exact release tag.

- Omit the version option to use the latest release.
- Without a directory option, the installer uses the default directory described above.
- For a custom directory, append `--install-dir "/path/to/bin"` to the sh command or `-InstallDir "C:\Tools\mdv"` to the PowerShell command.
- `MDV_INSTALL_DIR` also sets the sh install directory.
- PowerShell's optional `-NoModifyPath` leaves both the user and current-session PATH unchanged.

### Download before running

To inspect the script before running it, save it locally and then pass the same options:

```sh
curl -fsSL https://whosowsee.github.io/mdv/install.sh -o install.sh
sh install.sh --version 'v<VERSION>'
```

```powershell
irm https://whosowsee.github.io/mdv/install.ps1 -OutFile install.ps1
& ([scriptblock]::Create((Get-Content -Raw ./install.ps1))) -Version 'v<VERSION>'
```

A missing asset, digest or failed verification stops installation before the existing binary is replaced. These scripts do not install shell completions or file associations. To uninstall, remove the installed binary and any PATH entry you added.

## Install with Cargo

With Rust 1.95 or newer and Cargo installed:

```sh
cargo install mdv
```

Cargo builds the published release and installs the executable into its binary directory. Make sure that directory is on your `PATH`.

## Download a release

Download an archive for your operating system and architecture from [GitHub Releases](https://github.com/WhoSowSee/mdv/releases). Extract it and put the executable on your `PATH`.

Ready-made binaries do not require the Rust toolchain. Releases also include Linux packages; choose the package and architecture that match your system. RPM packages require glibc 2.39 or newer.

## Install RPM or Arch Linux packages

These commands download the package from [GitHub Releases](https://github.com/WhoSowSee/mdv/releases) into the current directory, then install it. `curl` is required. Run the installation line after the download finishes successfully. You can also download a package manually and run only the second line.

For Fedora and compatible RPM systems (x86_64):

```sh
curl -fLO "https://github.com/WhoSowSee/mdv/releases/download/v<VERSION>/mdv-v<VERSION>-x86_64-unknown-linux-gnu.rpm"
sudo dnf install "./mdv-v<VERSION>-x86_64-unknown-linux-gnu.rpm"
```

For Arch Linux (x86_64):

```sh
curl -fLO "https://github.com/WhoSowSee/mdv/releases/download/v<VERSION>/mdv-v<VERSION>-x86_64-unknown-linux-gnu.pkg.tar.zst"
sudo pacman -U "./mdv-v<VERSION>-x86_64-unknown-linux-gnu.pkg.tar.zst"
```

Replace every `<VERSION>` with the release version.

- For ARM64 RPM packages, replace `x86_64` with `aarch64` in both lines.
- Both package formats require glibc 2.39 or newer; Arch Linux ARM packages are not published.
- Rust is not required.
- Installing a local package does not register an update repository; install a new release package to update.

The homepage supplies exact filenames and package links from the latest release checked during its build. These package commands do not use the shell installer.

## Install with Nix

With Nix and flakes enabled, try mdv without a persistent install:

```sh
nix run github:WhoSowSee/mdv
```

Or install it into your profile:

```sh
nix profile install github:WhoSowSee/mdv
```

The flake supports x86_64 and ARM64 Linux.

## Build from source

With Git and Rust 1.95 or newer:

```sh
git clone https://github.com/WhoSowSee/mdv.git
cd mdv
cargo install --path .
```

## Check your installation

```sh
mdv --version
mdv README.md
```

Use a terminal with ANSI color support. A Nerd Font is needed only when you enable features that display Nerd Font icons.

## Run without installing

```bash
cargo build --release
./target/release/mdv README.md
```

Use the `mdv` binary from `target/release` directly or add it to your PATH.
