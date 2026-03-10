# Rust examples workspace Makefile
#
# Build bundle:  make                (recommended)
# Build plugins:  make build         (shared libraries for dynamic loading)
# Build file/net/exec:  make build-file | build-net | build-exec
# Build bundles by type:  make bundle-file | bundle-net | bundle-exec
# Build dist:  make dist             (static archives for bundling)
# Clean:  make clean
# Dev container:  make dev

LOCAL_SDK := ..
CARGO_CONFIG := .cargo/config.toml
FILE_PLUGINS := file-logger access-control byte-counter content-filter text-transform
EXEC_PLUGINS := exec-logger
NET_PLUGINS := net-logger net-transform
PLUGINS := $(FILE_PLUGINS) $(EXEC_PLUGINS) $(NET_PLUGINS)
IMAGE_NAME := qcontrol-sdk-rust-demo

.PHONY: all bundle bundle-file bundle-exec bundle-net setup-sdk build build-file build-exec build-net dist dist-file dist-exec dist-net clean docker-build dev

all: bundle

bundle: setup-sdk
	qcontrol bundle --config bundle.toml

bundle-file: setup-sdk
	qcontrol bundle --config bundle-file.toml

bundle-exec: setup-sdk
	qcontrol bundle --config bundle-exec.toml

bundle-net: setup-sdk
	qcontrol bundle --config bundle-net.toml

# Auto-generate .cargo/config.toml if local SDK exists
setup-sdk:
	@if [ -d "$(LOCAL_SDK)" ]; then \
		mkdir -p .cargo; \
		echo "# Auto-generated - local SDK override for monorepo development" > $(CARGO_CONFIG); \
		echo '[patch."https://github.com/qpoint-io/qcontrol-sdk-rust"]' >> $(CARGO_CONFIG); \
		echo 'qcontrol = { path = "$(LOCAL_SDK)" }' >> $(CARGO_CONFIG); \
		echo '[patch."git+https://github.com/qpoint-io/qcontrol-sdk-rust"]' >> $(CARGO_CONFIG); \
		echo 'qcontrol = { path = "$(LOCAL_SDK)" }' >> $(CARGO_CONFIG); \
		echo '[patch."ssh://git@github.com/qpoint-io/qcontrol-sdk-rust"]' >> $(CARGO_CONFIG); \
		echo 'qcontrol = { path = "$(LOCAL_SDK)" }' >> $(CARGO_CONFIG); \
		echo "Using local SDK at $(LOCAL_SDK)"; \
	elif [ -f "$(CARGO_CONFIG)" ]; then \
		rm $(CARGO_CONFIG); \
		echo "Local SDK not found, using git dependency"; \
	fi

build: build-file build-exec build-net

build-file:
	@for p in $(FILE_PLUGINS); do $(MAKE) -C $$p build; done

build-exec:
	@for p in $(EXEC_PLUGINS); do $(MAKE) -C $$p build; done

build-net:
	@for p in $(NET_PLUGINS); do $(MAKE) -C $$p build; done

dist: dist-file dist-exec dist-net

dist-file:
	@for p in $(FILE_PLUGINS); do $(MAKE) -C $$p dist; done

dist-exec:
	@for p in $(EXEC_PLUGINS); do $(MAKE) -C $$p dist; done

dist-net:
	@for p in $(NET_PLUGINS); do $(MAKE) -C $$p dist; done

clean:
	rm -f rust-plugins.so rust-file-plugins.so rust-exec-plugins.so rust-net-plugins.so
	cargo clean
	rm -rf .cargo
	@for p in $(PLUGINS); do $(MAKE) -C $$p clean; done

docker-build:
	docker build -t $(IMAGE_NAME) -f Dockerfile ..

dev: docker-build
	docker run --rm -it $(IMAGE_NAME)
