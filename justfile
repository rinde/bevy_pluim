# Common commands for rin-bevy-ui.
# Run `just` on its own to see this list.

set shell := ["bash", "-uc"]

# Bevy's own linter lags Bevy releases and needs a pinned nightly; see the
# `bevy-lint` recipe. v0.4.0 is the release that targets Bevy 0.16.
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

# Run the whole test suite, doc tests included.
test *ARGS:
    cargo test --quiet --all-targets {{ARGS}}
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

# Everything CI would run.
ci: lint test unused-deps

# Build and open the API documentation.
doc:
    cargo doc --no-deps --open

# Remove build artefacts.
clean:
    cargo clean
