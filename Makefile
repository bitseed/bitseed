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
	cargo test --test '*'

# Target for running E2E tests
e2e_test:
	cd tests/e2e && cargo test

# Target for running all tests (unit, integration, and E2E)
test: unit_test integration_test e2e_test

# Target for cleaning the project
clean:
	cargo clean

# Default target
.PHONY: default
default: build