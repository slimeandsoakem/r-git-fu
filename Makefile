FU_TEST_REPO := $$(pwd)
PREFIX ?= /usr/local
BINDIR := $(PREFIX)/bin

# Name of the binary
BINARY := r-git-fu

# Build flags
CARGO := cargo
PROFILE := release

# Default target: build release
all: build

# Build the project in release mode
build:
	@echo "Building $(BINARY) in release mode..."
	$(CARGO) build --release

# Run tests
test:
	@echo "Running tests..."
	FU_TEST_REPO=$(FU_TEST_REPO) $(CARGO) test

# Install the binary
install: build
	@echo "Installing $(BINARY) to $(BINDIR)..."
	install -d $(BINDIR)
	install -m 755 target/release/$(BINARY) $(BINDIR)/$(BINARY)

# Clean build artifacts
clean:
	$(CARGO) clean

# Uninstall the binary
uninstall:
	@echo "Removing $(BINDIR)/$(BINARY)..."
	rm -f $(BINDIR)/$(BINARY)

.PHONY: all build test install clean uninstall
