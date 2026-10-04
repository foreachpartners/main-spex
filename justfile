default:
    @just --list

lint:
    cargo check

fmt: fmt-docs
    cargo fmt

test:
    cargo test

fmt-docs:
    dprint fmt

check-docs:
    dprint check

install:
    cargo install --path . --force

help:
    spex --help
