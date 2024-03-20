# Define the project name and source code directory
PROJECT_NAME := bitseed
SRC_DIR := src

# Target for building the project
build:
	cargo build

# Target for running unit tests
unit-test:
	cargo test --lib

# Target for running integration tests
integration-test:
	RUST_LOG=debug cargo test --test '*'

# Target for running all tests (unit and integration)
test: unit-test integration-test

# Target for cleaning the project
clean:
	cargo clean

# Default target
.PHONY: default
default: build