# Common commands for bevy-pluim.
# Run `just` on its own to see this list.

set shell := ["bash", "-uc"]

# Compile through sccache when it is installed, so worktrees reuse each other's
# dependency builds without sharing a `target/`. A RUSTC_WRAPPER already set
# wins, and an empty one turns the wrapper off.
export RUSTC_WRAPPER := env_var_or_default("RUSTC_WRAPPER", `command -v sccache || true`)

# Bevy's own linter lags Bevy releases and needs a pinned nightly; see the
# `bevy-lint` recipe. v0.4.0 targets Bevy 0.16, so these pins are stale
# since the move to Bevy 0.17: bump them to the release that targets it
# before running the recipe.
BEVY_LINT_VERSION := "v0.4.0"
BEVY_LINT_TOOLCHAIN := "nightly-2025-06-26"

# The toolchain channel pinned in rust-toolchain.toml, so CI installs the
# same version instead of hard-coding it separately.
rust_version := `sed -nr 's/channel = "(.*)"/\1/p' rust-toolchain.toml`

_default:
    @just --list --unsorted

# Print the pinned Rust toolchain version (used by CI).
rust-version:
    echo '{{rust_version}}'

# Build the library, tests and examples.
build:
    cargo build --quiet --all-targets

# Type check without producing binaries.
check:
    cargo check --quiet --all-targets

# Run one of the examples, e.g. `just example widgets`.
example NAME *ARGS:
    cargo run --example {{NAME}} -- {{ARGS}}

# Play every scenario of tests/ron in the gallery's real window, one by one,
# and exit non-zero, naming what failed, if any did. Needs a display; not part
# of `just ci`.
gallery-test:
    cargo run --quiet --example gallery -- --run-all

# Run the whole test suite, doc tests included.
test *ARGS:
    cargo nextest run --all-targets {{ARGS}}
    cargo test --quiet --doc

# Check formatting and run clippy at the project's settings.
lint: fmt-check
    cargo clippy --all-targets --locked --quiet -- -D warnings
    export RUSTDOCFLAGS="-D warnings"; cargo doc --no-deps --document-private-items

# Reformat everything.
fmt:
    cargo fmt --all

# Check formatting without changing anything.
fmt-check:
    cargo fmt --all --check

# Report dependencies that are declared but never used.
unused-deps:
    # `--with-metadata` resolves renamed crates, which is the accurate mode.
    # Anything deliberately unused is listed in Cargo.toml under
    # `package.metadata.cargo-machete`.
    cargo machete --with-metadata

# Run Bevy's own linter, which catches mistakes clippy cannot see.
bevy-lint:
    # bevy_lint hooks into compiler internals, so it needs its own pinned
    # nightly, and each release targets one Bevy version. Check that
    # {{BEVY_LINT_VERSION}} supports the Bevy version in Cargo.toml before
    # trusting the output. That coupling is why this is not part of `just ci`.
    rustup toolchain install {{BEVY_LINT_TOOLCHAIN}} --component rustc-dev --component llvm-tools
    rustup run {{BEVY_LINT_TOOLCHAIN}} cargo install \
        --git https://github.com/TheBevyFlock/bevy_cli.git \
        --tag lint-{{BEVY_LINT_VERSION}} --locked bevy_lint
    bevy_lint --all-targets

# Record every scenario of tests/ron offscreen for review: PNGs, a filmstrip and
# a GIF per scenario, a trace.html, and an index.html per scenario,
# in target/recordings (RECORD_DIR overrides it). Needs a GPU, or lavapipe with
# WGPU_BACKEND=vulkan; not part of `just ci`.
record:
    cargo nextest run --test recorded_scenarios_gpu --features record --run-ignored all
    @if [ -z "${RECORD_CHECK_ONLY+x}" ]; then echo "recordings are in ${RECORD_DIR:-target/recordings}/<scenario>/index.html"; fi

# Everything CI would run.
ci: lint test unused-deps

# Build and open the API documentation.
doc:
    cargo doc --no-deps --open

# Remove build artefacts.
clean:
    cargo clean
