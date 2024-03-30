PROJECT_NAME := bitseed
SRC_DIR := src

# Target for building the project
build:
	cargo build

# Target for running unit tests
unit-test:
	RUST_LOG=debug RUST_BACKTRACE=full cargo test --lib

# Target for running integration tests
integration-test:
	RUST_LOG=debug RUST_BACKTRACE=full cargo test --test '*'

# Target for running all tests (unit and integration)
test: unit-test integration-test

# Target for cleaning the project
clean:
	cargo clean

# Default target
.PHONY: default
default: build