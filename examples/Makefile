# Rust examples workspace Makefile
#
# Build bundle:  make           (recommended)
# Build plugins:  make build    (shared libraries for dynamic loading)
# Build dist:  make dist        (static archives for bundling)
# Clean:  make clean
# Dev container:  make dev

LOCAL_SDK := ..
CARGO_CONFIG := .cargo/config.toml
IMAGE_NAME := qcontrol-sdk-rust-demo

.PHONY: all bundle setup-sdk build dist clean docker-build dev

all: bundle

bundle: setup-sdk
	qcontrol bundle --config bundle.toml

# Auto-generate .cargo/config.toml if local SDK exists
# On macOS, adds -undefined dynamic_lookup so plugins can reference
# qcontrol runtime symbols that are resolved at load time.
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
	@if [ "$$(uname -s)" = "Darwin" ]; then \
		mkdir -p .cargo; \
		echo '' >> $(CARGO_CONFIG); \
		echo '[target.aarch64-apple-darwin]' >> $(CARGO_CONFIG); \
		echo 'rustflags = ["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]' >> $(CARGO_CONFIG); \
		echo '' >> $(CARGO_CONFIG); \
		echo '[target.x86_64-apple-darwin]' >> $(CARGO_CONFIG); \
		echo 'rustflags = ["-C", "link-arg=-undefined", "-C", "link-arg=dynamic_lookup"]' >> $(CARGO_CONFIG); \
		echo "Added macOS linker flags for plugin symbol resolution"; \
	fi

build: setup-sdk
	cargo build --release

dist: setup-sdk
	cargo build --release

clean:
	rm -f rust-plugins.so
	cargo clean
	rm -rf .cargo
	rm -rf file-logger/dist access-control/dist byte-counter/dist content-filter/dist text-transform/dist

docker-build:
	docker build -t $(IMAGE_NAME) -f Dockerfile ..

dev: docker-build
	docker run --rm -it $(IMAGE_NAME)
