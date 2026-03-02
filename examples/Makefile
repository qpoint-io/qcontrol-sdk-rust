# Rust examples workspace Makefile
#
# SDK Detection: Auto-generates .cargo/config.toml to use local SDK in monorepo
# Build:         make build (builds all plugins)
# Clean:         make clean (removes artifacts and config)

LOCAL_SDK := ..
CARGO_CONFIG := .cargo/config.toml

.PHONY: all setup-sdk build dist clean

all: build

# Auto-generate .cargo/config.toml if local SDK exists
setup-sdk:
	@if [ -d "$(LOCAL_SDK)" ]; then \
		mkdir -p .cargo; \
		echo "# Auto-generated - local SDK override for monorepo development" > $(CARGO_CONFIG); \
		echo '[patch."https://github.com/qpoint-io/qcontrol-sdk-rust"]' >> $(CARGO_CONFIG); \
		echo 'qcontrol = { path = "$(LOCAL_SDK)" }' >> $(CARGO_CONFIG); \
		echo "Using local SDK at $(LOCAL_SDK)"; \
	elif [ -f "$(CARGO_CONFIG)" ]; then \
		rm $(CARGO_CONFIG); \
		echo "Local SDK not found, using git dependency"; \
	fi

build: setup-sdk
	cargo build --release

dist: setup-sdk
	cargo build --release

clean:
	cargo clean
	rm -rf .cargo
	rm -rf file-logger/dist access-control/dist byte-counter/dist content-filter/dist text-transform/dist
