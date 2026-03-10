# byte-counter plugin Makefile
#
# Build:   make build  (shared library for dynamic loading)
# Dist:    make dist   (static archive for bundling)
# Clean:   make clean

NAME := byte-counter
ARCH := $(shell uname -m | sed 's/^arm64$$/aarch64/')-$(shell uname -s | tr A-Z a-z | sed 's/^darwin$$/macos/')
DIST := dist
SHLIB_EXT := $(if $(filter Darwin,$(shell uname -s)),dylib,so)

# Rust library names use underscores
LIB_NAME := byte_counter

.PHONY: all build dist clean setup-sdk

all: build

# Ensure local SDK is configured before building
setup-sdk:
	@$(MAKE) -C .. setup-sdk

build: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).$(SHLIB_EXT) $(DIST)/$(NAME)-$(ARCH).$(SHLIB_EXT)

dist: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).a $(DIST)/$(NAME)-$(ARCH).a

clean:
	rm -rf $(DIST)
