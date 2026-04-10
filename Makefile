.PHONY: dev build test lint fmt migrate clean docker

# ─── Development ───
dev:
	cargo watch -x 'run -- --config configs/default.toml'

build:
	cargo build --release

# ─── Testing ───
test:
	cargo test -- --nocapture

test-unit:
	cargo test --lib -- --nocapture

test-integration:
	cargo test --test '*' -- --nocapture

# ─── Linting ───
lint:
	cargo clippy --all-targets -- -D warnings

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all -- --check

# ─── Database ───
migrate:
	sqlx migrate run --source migrations/

migrate-create:
	@read -p "Migration name: " name; \
	sqlx migrate add -r $$name --source migrations/

# ─── Docker ───
docker:
	docker build -t nexus-gate:latest .

docker-up:
	docker compose up -d

docker-down:
	docker compose down

docker-prod:
	docker compose -f docker-compose.prod.yml up -d --build

docker-prod-down:
	docker compose -f docker-compose.prod.yml down

# ─── Deploy verification ───
verify:
	./scripts/verify-deploy.sh

verify-remote:
	@read -p "Enter URL (e.g. https://gate.example.com): " url; \
	./scripts/verify-deploy.sh $$url

# ─── Cleanup ───
clean:
	cargo clean
	rm -rf dist/

# ─── All checks ───
check: fmt-check lint test
	@echo "All checks passed"
