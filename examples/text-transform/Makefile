# text-transform plugin Makefile
#
# Build:   make build  (shared library for dynamic loading)
# Dist:    make dist   (static archive for bundling)
# Clean:   make clean

NAME := text-transform
# Normalize arch/OS to match Rust std::env::consts (used by qcontrol bundler)
RAW_ARCH := $(shell uname -m)
RAW_OS := $(shell uname -s | tr A-Z a-z)
NORM_ARCH := $(if $(filter arm64,$(RAW_ARCH)),aarch64,$(RAW_ARCH))
NORM_OS := $(if $(filter darwin,$(RAW_OS)),macos,$(RAW_OS))
ARCH := $(NORM_ARCH)-$(NORM_OS)
DIST := dist

# Rust library names use underscores
LIB_NAME := text_transform

.PHONY: all build dist clean setup-sdk

all: build

# Ensure local SDK is configured before building
setup-sdk:
	@$(MAKE) -C .. setup-sdk

EXT := $(if $(filter macos,$(NORM_OS)),dylib,so)

build: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).$(EXT) $(DIST)/$(NAME)-$(ARCH).$(EXT)

dist: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).a $(DIST)/$(NAME)-$(ARCH).a

clean:
	rm -rf $(DIST)
