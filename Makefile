# norn project entry points. CI runs the same gates.

.PHONY: build check fmt clippy test install link smoke

build: ## cargo build (workspace, release)
	cargo build --release --locked

fmt: ## cargo fmt --check
	cargo fmt --all --check

clippy: ## clippy, warnings denied
	cargo clippy --all-targets -- -D warnings

test: ## cargo test (workspace)
	cargo test --locked

check: fmt clippy test ## all gates, CI order

install: build ## install the binaries into the plugin root's bin/
	install -d bin
	install -m 0755 target/release/norn target/release/skald target/release/saga bin/

link: install ## herdr plugin link (dev install from this checkout)
	herdr plugin link "$(CURDIR)"

smoke: ## one capture pass + a read-back, against the default session
	NORN_POLL_SECS=1 target/release/norn record & pid=$$!; sleep 6; kill $$pid; \
	target/release/skald projects && target/release/saga projects
