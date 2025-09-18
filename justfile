set shell := ["bash", "-cu"]

default: build

build:
	cargo build

run:
	cargo run -p api

migrate:
	SQLX_OFFLINE=true sqlx migrate run

prepare:
	sqlx prepare --workspace -- --all-features

up:
	docker compose up -d --wait

down:
	docker compose down -v

test:
	cargo test

check:
	cargo check --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt

clean:
	cargo clean
