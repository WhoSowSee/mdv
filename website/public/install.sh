#!/bin/sh

mdv_fail() {
    printf 'mdv installer: %s\n' "$*" >&2
    exit 1
}

mdv_release_field() {
    awk -v mode="$1" -v wanted_asset="${2:-}" '
    {
        for (i = 1; i <= length($0); i++) {
            c = substr($0, i, 1)
            if (in_string) {
                if (escaped) {
                    text = text c
                    escaped = 0
                } else if (c == "\\") {
                    escaped = 1
                } else if (c == "\"") {
                    in_string = 0
                    if (value) {
                        if (depth == 1 && key == "tag_name") {
                            tag = text
                            tag_count++
                        }
                        if (asset_depth && depth == asset_depth) {
                            if (key == "name") asset_name = text
                            if (key == "digest") asset_digest = text
                        }
                        value = 0
                    } else {
                        last_string = text
                    }
                } else {
                    text = text c
                }
                continue
            }
            if (c == "\"") {
                in_string = 1
                text = ""
            } else if (c == ":") {
                key = last_string
                value = 1
            } else if (c == "{") {
                depth++
                value = 0
                if (assets_depth && depth == assets_depth + 1) {
                    asset_depth = depth
                    asset_name = asset_digest = ""
                }
            } else if (c == "}") {
                if (asset_depth && depth == asset_depth) {
                    if (asset_name == wanted_asset) {
                        digest = asset_digest
                        asset_count++
                    }
                    asset_depth = 0
                }
                depth--
                if (depth < 0) invalid = 1
                value = 0
            } else if (c == "[") {
                if (depth == 1 && key == "assets" && value) assets_depth = depth
                value = 0
            } else if (c == "]") {
                if (depth == assets_depth) assets_depth = 0
                value = 0
            } else if (c == ",") {
                value = 0
                key = ""
            }
        }
    }
    END {
        if (invalid || in_string || depth != 0) exit 1
        if (mode == "tag" && tag_count == 1) print tag
        else if (mode == "digest" && asset_count == 1 && digest != "") print digest
        else exit 1
    }' "$mdv_work_dir/release.json"
}

mdv_download() {
    curl --fail --silent --show-error --location --proto '=https' --tlsv1.2 \
        --header 'Accept: application/vnd.github+json' \
        --header 'User-Agent: mdv-installer' --output "$2" "$1"
}

mdv_cleanup() {
    if [ -n "$mdv_staged_file" ]; then rm -f "$mdv_staged_file"; fi
    case "$mdv_work_dir" in
        "$mdv_temp_root"/mdv-install.*) rm -rf "$mdv_work_dir" ;;
    esac
}

mdv_install() {
    set -eu
    mdv_version=latest
    mdv_install_dir=${MDV_INSTALL_DIR:-}
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --version|--install-dir)
                [ "$#" -ge 2 ] || mdv_fail "Missing value for $1"
                case "$1" in
                    --version) mdv_version=$2 ;;
                    --install-dir) mdv_install_dir=$2 ;;
                esac
                shift 2
                ;;
            --help|-h)
                printf '%s\n' 'Usage: install.sh [--version TAG] [--install-dir DIRECTORY]' \
                    'Defaults: latest release, $HOME/.local/bin (or MDV_INSTALL_DIR).' \
                    'Supports Linux x86_64/ARM64 and macOS Intel/Apple Silicon.'
                return 0
                ;;
            *) mdv_fail "Unknown argument: $1" ;;
        esac
    done

    case "$mdv_version" in ''|[!A-Za-z0-9]*|*[!A-Za-z0-9._+-]*) mdv_fail 'Invalid release tag' ;; esac
    [ -z "${TERMUX_VERSION:-}" ] || mdv_fail 'Android releases are not available'
    for mdv_tool in curl uname awk mktemp unzip chmod mkdir mv rm; do
        command -v "$mdv_tool" >/dev/null 2>&1 || mdv_fail "Required command not found: $mdv_tool"
    done
    mdv_os=$(uname -s)
    mdv_arch=$(uname -m)
    case "$mdv_os:$mdv_arch" in
        Linux:x86_64|Linux:amd64) mdv_target=x86_64-unknown-linux-musl ;;
        Linux:aarch64|Linux:arm64) mdv_target=aarch64-unknown-linux-musl ;;
        Darwin:x86_64|Darwin:amd64) mdv_target=x86_64-apple-darwin ;;
        Darwin:arm64|Darwin:aarch64) mdv_target=aarch64-apple-darwin ;;
        *) mdv_fail "Unsupported platform: $mdv_os $mdv_arch" ;;
    esac
    case "$mdv_os" in
        Linux) mdv_hash_tool=sha256sum ;;
        Darwin) mdv_hash_tool=shasum ;;
    esac
    command -v "$mdv_hash_tool" >/dev/null 2>&1 || mdv_fail "Required command not found: $mdv_hash_tool"

    if [ -z "$mdv_install_dir" ]; then
        [ -n "${HOME:-}" ] || mdv_fail 'HOME is unset; specify --install-dir'
        mdv_install_dir="$HOME/.local/bin"
    fi
    case "$mdv_install_dir" in /*) ;; *) mdv_install_dir="$(pwd)/$mdv_install_dir" ;; esac
    mdv_destination="$mdv_install_dir/mdv"
    [ ! -d "$mdv_destination" ] || mdv_fail "Destination is a directory: $mdv_destination"

    mdv_temp_root=${TMPDIR:-/tmp}
    case "$mdv_temp_root" in /*) ;; *) mdv_fail 'TMPDIR must be an absolute path' ;; esac
    mdv_work_dir=$(mktemp -d "$mdv_temp_root/mdv-install.XXXXXXXX")
    mdv_staged_file=
    trap mdv_cleanup EXIT
    trap 'exit 1' HUP INT TERM

    mdv_api=https://api.github.com/repos/WhoSowSee/mdv/releases
    case "$mdv_version" in
        latest) mdv_release_url="$mdv_api/latest" ;;
        *) mdv_release_url="$mdv_api/tags/$mdv_version" ;;
    esac
    mdv_download "$mdv_release_url" "$mdv_work_dir/release.json"
    mdv_tag=$(mdv_release_field tag) || mdv_fail 'Cannot read the release tag'
    case "$mdv_tag" in ''|*[!A-Za-z0-9._+-]*) mdv_fail 'Invalid release tag in GitHub response' ;; esac
    mdv_asset="mdv-$mdv_tag-$mdv_target.zip"
    mdv_digest=$(mdv_release_field digest "$mdv_asset") || mdv_fail "Missing unique asset or SHA-256 digest: $mdv_asset"
    case "$mdv_digest" in sha256:*) mdv_expected=${mdv_digest#sha256:} ;; *) mdv_fail 'Missing SHA-256 digest' ;; esac
    [ "${#mdv_expected}" -eq 64 ] || mdv_fail 'Invalid SHA-256 digest'
    case "$mdv_expected" in *[!a-f0-9]*) mdv_fail 'Invalid SHA-256 digest' ;; esac

    mdv_archive="$mdv_work_dir/mdv.zip"
    printf 'Downloading %s...\n' "$mdv_asset"
    mdv_download "https://github.com/WhoSowSee/mdv/releases/download/$mdv_tag/$mdv_asset" "$mdv_archive"
    case "$mdv_os" in
        Linux) mdv_actual=$(sha256sum "$mdv_archive" | awk '{print $1}') ;;
        Darwin) mdv_actual=$(shasum -a 256 "$mdv_archive" | awk '{print $1}') ;;
    esac
    [ "$mdv_actual" = "$mdv_expected" ] || mdv_fail 'SHA-256 mismatch; existing installation was not changed'
    mdv_entries=$(unzip -Z1 "$mdv_archive") || mdv_fail 'Cannot read release archive'
    mdv_entry_count=$(printf '%s\n' "$mdv_entries" | awk '$0 == "mdv" {n++} END {print n+0}')
    [ "$mdv_entry_count" -eq 1 ] || mdv_fail 'Archive must contain exactly one root-level mdv binary'

    mkdir -p "$mdv_install_dir"
    mdv_staged_file=$(mktemp "$mdv_install_dir/.mdv.XXXXXXXX")
    unzip -p "$mdv_archive" mdv > "$mdv_staged_file"
    [ -s "$mdv_staged_file" ] || mdv_fail 'Release binary is empty'
    chmod 755 "$mdv_staged_file"
    mv -f "$mdv_staged_file" "$mdv_destination"
    mdv_staged_file=
    printf 'Installed mdv %s at %s\n' "${mdv_tag#v}" "$mdv_destination"
    case ":$PATH:" in
        *":$mdv_install_dir:"*) ;;
        *)
            printf 'Add the installation directory to PATH: %s\n' "$mdv_install_dir"
            if [ "$mdv_install_dir" = "${HOME:-}/.local/bin" ]; then
                printf '%s\n' 'For the current shell: export PATH="$HOME/.local/bin:$PATH"'
            fi
            ;;
    esac
}

mdv_install "$@"
