.PHONY: build install uninstall test clean run help

# Default install directory
INSTALL_DIR ?= $(HOME)/.local/bin
BINARY_NAME = git-nexus

help: ## Show this help message
	@echo "git-nexus - A blazing fast multi-repository scanner"
	@echo ""
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in release mode
	@echo "📦 Building $(BINARY_NAME)..."
	@cargo build --release
	@echo "✅ Build complete: target/release/$(BINARY_NAME)"

install: build ## Build and install to ~/.local/bin (or custom INSTALL_DIR)
	@echo "📥 Installing to $(INSTALL_DIR)..."
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@chmod +x $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✅ Installed to $(INSTALL_DIR)/$(BINARY_NAME)"

uninstall: ## Uninstall from ~/.local/bin (or custom INSTALL_DIR)
	@echo "🗑️  Uninstalling $(BINARY_NAME)..."
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "✅ Uninstalled"

test: ## Run tests
	@echo "🧪 Running tests..."
	@cargo test

clean: ## Clean build artifacts
	@echo "🧹 Cleaning..."
	@cargo clean
	@echo "✅ Clean complete"

run: ## Run the project (development mode)
	@cargo run -- .

run-verbose: ## Run with verbose output
	@cargo run -- . -v

run-json: ## Run with JSON output
	@cargo run -- . --json -v

dev: ## Build and run in development mode
	@cargo build && cargo run -- .

check: ## Check the project for errors
	@cargo check

fmt: ## Format the code
	@cargo fmt

clippy: ## Run clippy lints
	@cargo clippy -- -D warnings

all: clean build test ## Clean, build, and test

# Custom install directory example:
# make install INSTALL_DIR=/usr/local/bin
