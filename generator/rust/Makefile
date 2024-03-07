# Set our target architecture to wasm32-unknown-unknown
TARGET = wasm32-unknown-unknown
# Set the release compilation flag
RELEASE_FLAG = --release
# Set our crate type to cdylib
CRATE_TYPE = cdylib
# Set the output directory
OUT_DIR = ./out
# Set the optimization level for the wasm-opt tool
OPT_LEVEL = -Oz
# Set the name of our wasm file
WASM_FILE_NAME = rust

# Default target
all: build

# Dependency installation target
deps:
	@echo "Installing dependencies"
	rustup target add $(TARGET)
	cargo install wasm-bindgen-cli
	# The following line assumes you have binaryen installed or available in your package manager
	# For example, on Ubuntu you might use: sudo apt-get install binaryen
	# On macOS with Homebrew: brew install binaryen
	# Please adjust the command according to your package manager or install binaryen manually
	# sudo apt-get install binaryen

# Build target
build:
	@echo "Building Rust to WASM with no_std"
	RUSTFLAGS='-C link-arg=-s' cargo build $(RELEASE_FLAG) --target $(TARGET) --no-default-features
	wasm-bindgen --no-modules --no-typescript --out-dir $(OUT_DIR) --target web target/$(TARGET)/release/$(WASM_FILE_NAME).wasm
	wasm-opt $(OPT_LEVEL) -o $(OUT_DIR)/$(WASM_FILE_NAME)_optimized.wasm $(OUT_DIR)/$(WASM_FILE_NAME)_bg.wasm

# Clean target
clean:
	@echo "Cleaning up"
	cargo clean
	rm -rf $(OUT_DIR)

# Set .PHONY to indicate to make that these are not file names
.PHONY: all deps build clean

