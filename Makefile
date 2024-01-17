#!/bin/bash

build-evn=SLINT_STYLE=fluent
run-evn=RUST_LOG=error,warn,info,debug,sqlx=off,reqwest=off

all:
	$(build-evn) cargo build --release

build:
	$(build-evn) cargo build --release

build-debug:
	$(build-evn) cargo build

run:
	$(build-evn) $(run-evn) cargo run

run-local-debug:
	$(run-evn) ./target/debug/vtbox

run-local-release:
	$(run-evn) ./target/release/vtbox

test:
	$(build-evn) $(run-evn) cargo test -- --nocapture

clippy:
	cargo clippy

clean-incremental:
	rm -rf ./target/debug/incremental/*

clean:
	cargo clean

install:
	cp -rf ./target/release/vtbox ~/bin/

slint-view:
	slint-viewer --style fluent --auto-reload -I vtbox/ui ./vtbox/ui/appwindow.slint
