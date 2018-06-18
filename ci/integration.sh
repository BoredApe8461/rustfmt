#!/usr/bin/env bash

set -ex

: ${INTEGRATION?"The INTEGRATION environment variable must be set."}

# FIXME: this means we can get a stale cargo-fmt from a previous run.
#
# `which rustfmt` fails if rustfmt is not found. Since we don't install
# `rustfmt` via `rustup`, this is the case unless we manually install it. Once
# that happens, `cargo install --force` will be called, which installs
# `rustfmt`, `cargo-fmt`, etc to `~/.cargo/bin`. This directory is cached by
# travis (see `.travis.yml`'s "cache" key), such that build-bots that arrive
# here after the first installation will find `rustfmt` and won't need to build
# it again.
#
#which cargo-fmt || cargo install --force
cargo install --force

echo "Integration tests for: ${INTEGRATION}"
cargo fmt -- --version

# Checks that:
#
# * `cargo fmt --all` succeeds without any warnings or errors
# * `cargo fmt --all -- --check` after formatting returns success
# * `cargo test -all` still passes (formatting did not break the build)
function check_fmt {
    cargo test --all
    if [[ $? != 0 ]]; then
          return 0
    fi
    touch rustfmt.toml
    cargo fmt --all -v |& tee rustfmt_output
    if [[ ${PIPESTATUS[0]} != 0 ]]; then
        cat rustfmt_output
        return 1
    fi
    cat rustfmt_output
    ! cat rustfmt_output | grep -q "internal error"
    if [[ $? != 0 ]]; then
        return 1
    fi
    ! cat rustfmt_output | grep -q "warning"
    if [[ $? != 0 ]]; then
        return 1
    fi
    ! cat rustfmt_output | grep -q "Warning"
    if [[ $? != 0 ]]; then
        return 1
    fi
    cargo fmt --all -- --check |& tee rustfmt_check_output
    if [[ ${PIPESTATUS[0]} != 0 ]]; then
        cat rustfmt_check_output
        return 1
    fi
    cargo test --all
    if [[ $? != 0 ]]; then
        return $?
    fi
}

case ${INTEGRATION} in
    cargo)
        git clone --depth=1 https://github.com/rust-lang/${INTEGRATION}.git
        cd ${INTEGRATION}
        export CFG_DISABLE_CROSS_TESTS=1
        check_fmt
        cd -
        ;;
    failure)
        git clone --depth=1 https://github.com/rust-lang-nursery/${INTEGRATION}.git
        cd ${INTEGRATION}/failure-1.X
        check_fmt
        cd -
        ;;
    *)
        git clone --depth=1 https://github.com/rust-lang-nursery/${INTEGRATION}.git
        cd ${INTEGRATION}
        check_fmt
        cd -
        ;;
esac
