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

# UI commands
ui-install:
	cd ui && pnpm install

ui-dev:
	cd ui && pnpm dev

ui-build:
	cd ui && pnpm build

# Run all services (API + UI)
dev:
	just up
	just migrate
	just run &
	just ui-dev
