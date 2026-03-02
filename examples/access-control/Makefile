# access-control plugin Makefile
#
# Build:   make build  (shared library for dynamic loading)
# Dist:    make dist   (static archive for bundling)
# Clean:   make clean

NAME := access-control
ARCH := $(shell uname -m)-$(shell uname -s | tr A-Z a-z)
DIST := dist

# Rust library names use underscores
LIB_NAME := access_control

.PHONY: all build dist clean setup-sdk

all: build

# Ensure local SDK is configured before building
setup-sdk:
	@$(MAKE) -C .. setup-sdk

build: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).so $(DIST)/$(NAME)-$(ARCH).so

dist: setup-sdk
	cargo build --release -p $(NAME)
	@mkdir -p $(DIST)
	cp ../target/release/lib$(LIB_NAME).a $(DIST)/$(NAME)-$(ARCH).a

clean:
	rm -rf $(DIST)
