default:
    @just --list

lint:
    cargo clippy --all-targets -- -D warnings

fmt: fmt-docs
    cargo fmt

test:
    cargo test

fmt-docs:
    dprint fmt

check-docs:
    dprint check

check-fmt: check-docs
    cargo fmt --check

install:
    cargo install --path . --force

help:
    spex --help
