CI := 1

.PHONY: check fmt clippy test deny security

# Check code formatting
fmt:
	cargo fmt --all -- --check

# Run clippy linter
clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run all tests
test:
	cargo test --workspace

# Check licenses and dependencies
deny:
	@command -v cargo-deny >/dev/null || (echo "Installing cargo-deny..." && cargo install cargo-deny)
	cargo deny check

# Run all security checks
security: deny

# Run all quality checks
check: fmt clippy test

# Show this help message
help:
	@awk '/^# / { desc=substr($$0, 3) } /^[a-zA-Z0-9_-]+:/ && desc { printf "%-20s - %s\n", $$1, desc; desc="" }' Makefile | sort
