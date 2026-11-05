#!/bin/bash
# Prepares a fresh container for working on this repository.
#
# Installs the Rust toolchain pinned in rust-toolchain.toml with clippy and
# rustfmt, `just` and `cargo-machete` at pinned versions, the system headers
# Bevy's default features need, then warms the build cache so the first real
# build is not a cold Bevy compile.
# Safe to run more than once: every step checks whether it already has what
# it needs.
#
# Runs automatically as a SessionStart hook in Claude Code on the web. Pass
# --force to run it by hand anywhere else.
set -euo pipefail

if [ "${1:-}" != "--force" ] && [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
    exit 0
fi

readonly REPO="${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)}"
cd "$REPO"

# Versions of the tools CI installs. Bump deliberately.
readonly JUST_VERSION="1.42.4"
readonly MACHETE_VERSION="0.9.1"

readonly BIN="${CARGO_HOME:-$HOME/.cargo}/bin"
mkdir -p "$BIN"

# Progress goes to stderr so it stays out of the session's context.
log() { printf '  %s\n' "$*" >&2; }

# --------------------------------------------------------------- environment

export PATH="$BIN:$PATH"
if [ -n "${CLAUDE_ENV_FILE:-}" ]; then
    printf 'export PATH=%q:$PATH\n' "$BIN" >>"$CLAUDE_ENV_FILE"
    echo 'export CARGO_TERM_COLOR=always' >>"$CLAUDE_ENV_FILE"
fi

# ---------------------------------------------------------------- toolchain

# `rust-toolchain.toml` pins the compiler, and rustup honours that pin for
# every cargo invocation in this directory whatever the default is. Read the
# channel rather than hard-coding it, so bumping the pin is a one-file change.
toolchain="$(sed -n 's/^channel[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' rust-toolchain.toml | head -1)"
if [ -z "$toolchain" ]; then
    echo "could not read channel from rust-toolchain.toml" >&2
    exit 1
fi

# rustup would install the pin on its own at the first build, but only with the
# default profile's components. `just lint` needs clippy and rustfmt, so ask
# for them here instead of discovering they are missing mid-lint.
log "installing the pinned toolchain $toolchain with clippy and rustfmt"
rustup toolchain install "$toolchain" --profile minimal \
    --component clippy --component rustfmt --no-self-update >&2

# ------------------------------------------------------------ system headers

# The default Bevy feature set (winit windowing, audio, gamepad) needs these
# to link, as in `.github/workflows/ci.yml`.
readonly BEVY_HEADERS=(pkg-config libx11-dev libasound2-dev libudev-dev libxkbcommon-x11-dev libwayland-dev)
if command -v apt-get >/dev/null 2>&1 && ! dpkg -s "${BEVY_HEADERS[@]}" >/dev/null 2>&1; then
    log "installing Bevy's system headers: ${BEVY_HEADERS[*]}"
    apt_install() { apt-get install -y --no-install-recommends "${BEVY_HEADERS[@]}" >&2; }
    if apt_install || { apt-get update >&2 && apt_install; }; then
        log "installed ${BEVY_HEADERS[*]}"
    else
        log "could not install Bevy's system headers; the build may fail to link"
    fi
else
    log "Bevy's system headers already installed"
fi

# --------------------------------------------------------------- utilities

# Prebuilt binaries come from GitHub releases. The egress policy allows those
# but blocks just.systems and api.github.com, so the URLs are spelled out and
# a crates.io build is the fallback.
install_tool() {
    local name="$1" version="$2" url="$3" crate="$4"
    if command -v "$name" >/dev/null 2>&1; then
        log "$name already installed ($("$name" --version 2>/dev/null | head -1))"
        return 0
    fi
    local tmp
    tmp="$(mktemp -d)"
    if curl -fsSL --retry 3 --connect-timeout 20 -o "$tmp/pkg.tar.gz" "$url" 2>/dev/null &&
        tar xzf "$tmp/pkg.tar.gz" -C "$tmp" 2>/dev/null; then
        local found
        found="$(find "$tmp" -type f -name "$name" -perm -u+x | head -1)"
        if [ -n "$found" ]; then
            install -m 0755 "$found" "$BIN/$name"
            log "$name $version installed"
            rm -rf "$tmp"
            return 0
        fi
    fi
    rm -rf "$tmp"
    log "no prebuilt $name; building it from crates.io instead"
    cargo install "$crate" --version "$version" --locked >&2
}

install_tool just "$JUST_VERSION" \
    "https://github.com/casey/just/releases/download/${JUST_VERSION}/just-${JUST_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    just

install_tool cargo-machete "$MACHETE_VERSION" \
    "https://github.com/bnjbvr/cargo-machete/releases/download/v${MACHETE_VERSION}/cargo-machete-v${MACHETE_VERSION}-x86_64-unknown-linux-musl.tar.gz" \
    cargo-machete

# ------------------------------------------------------------------ sources

log "fetching dependencies"
if ! cargo fetch --locked >&2; then
    log "could not fetch dependencies; check the Cargo and git errors above"
    exit 1
fi

# A cold build of the Bevy tree takes minutes. Doing it here warms the session's
# build cache before the first thing the agent runs.
# Best effort: a failure here must not stop the session from starting.
log "warming the build cache (a few minutes on a cold container)"
if ! cargo build --all-targets --locked >&2; then
    log "warm-up did not finish; the first build will just take longer"
fi

echo "bevy-pluim dev environment ready: rustc $(rustc --version | cut -d' ' -f2), $(just --version), cargo-machete $(cargo machete --version 2>/dev/null | head -1). Run 'just ci' for everything CI checks."
