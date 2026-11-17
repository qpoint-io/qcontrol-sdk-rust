# netstat-observe plugin Makefile
#
# Build:   make build  (shared library for dynamic loading)
# Clean:   make clean

NAME := netstat-observe
ARCH := $(shell uname -m | sed 's/^arm64$$/aarch64/')-$(shell uname -s | tr A-Z a-z | sed 's/^darwin$$/macos/')
DIST := dist
SHLIB_EXT := $(if $(filter Darwin,$(shell uname -s)),dylib,so)

LIB_NAME := netstat_observe

.PHONY: all build clean setup-sdk

all: build

setup-sdk:
	@$(MAKE) -C .. setup-sdk

build: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).$(SHLIB_EXT) $(DIST)/$(NAME)-$(ARCH).$(SHLIB_EXT)

clean:
	rm -rf $(DIST)
