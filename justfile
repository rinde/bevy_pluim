# Common commands for bevy-pluim.
# Run `just` on its own to see this list.

set shell := ["bash", "-uc"]

# Use sccache when installed; a RUSTC_WRAPPER you set wins.
export RUSTC_WRAPPER := env_var_or_default("RUSTC_WRAPPER", `command -v sccache || true`)

# Stale: v0.4.0 targets Bevy 0.16. Bump to the release for the current Bevy.
BEVY_LINT_VERSION := "v0.4.0"
BEVY_LINT_TOOLCHAIN := "nightly-2025-06-26"

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

# Run one of the examples, e.g. `just example overview`.
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
    cargo machete --with-metadata

# Run Bevy's own linter, which catches mistakes clippy cannot see.
bevy-lint:
    # Keep out of `just ci`: each bevy_lint release targets one Bevy version.
    rustup toolchain install {{BEVY_LINT_TOOLCHAIN}} --component rustc-dev --component llvm-tools
    rustup run {{BEVY_LINT_TOOLCHAIN}} cargo install \
        --git https://github.com/TheBevyFlock/bevy_cli.git \
        --tag lint-{{BEVY_LINT_VERSION}} --locked bevy_lint
    bevy_lint --all-targets

# Check rendered scenarios; --save writes recordings to RECORD_DIR. Needs a GPU.
record $option="":
    @case "$option" in "") RECORD_CHECK_ONLY=1 cargo nextest run --test recorded_scenarios_gpu --features record --run-ignored all ;; --save) unset RECORD_CHECK_ONLY; cargo nextest run --test recorded_scenarios_gpu --features record --run-ignored all && echo "recordings are in ${RECORD_DIR:-target/recordings}/<scenario>/index.html" ;; *) echo "usage: just record [--save]" >&2; exit 2 ;; esac

# Benchmark CPU time and memory, e.g. `just bench -- --workload tooltips --size 5000`.
bench *ARGS:
    cargo run --release --example bench -- {{ARGS}}

# Everything CI would run.
ci: lint test unused-deps

# Build and open the API documentation.
doc:
    cargo doc --no-deps --open

# Remove build artifacts.
clean:
    cargo clean
