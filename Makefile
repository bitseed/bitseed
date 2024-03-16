# Define the project name and source code directory
PROJECT_NAME := bitseed
SRC_DIR := src

# Target for building the project
build:
	cargo build

# Target for running unit tests
unit_test:
	cargo test --lib

# Target for running integration tests
integration_test:
	RUST_LOG=debug RUST_BACKTRACE=1 cargo test --test '*'

# Target for running all tests (unit and integration)
test: unit_test integration_test

# Target for cleaning the project
clean:
	cargo clean

# Default target
.PHONY: default
default: build