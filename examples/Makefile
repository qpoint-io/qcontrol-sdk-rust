# Rust examples workspace Makefile
#
# Build plugins:  make or make build
# Build file/net/exec:  make build-file | build-net | build-exec
# Clean:  make clean
# Dev container:  make dev

LOCAL_SDK := ..
CARGO_CONFIG := .cargo/config.toml
FILE_PLUGINS := file-logger access-control byte-counter content-filter text-transform
EXEC_PLUGINS := exec-logger
NET_PLUGINS := net-logger net-transform
NETSTAT_PLUGINS := netstat-observe
HTTP_PLUGINS := http-logger http-rewrite sse-rewrite websocket-rewrite
PLUGINS := $(FILE_PLUGINS) $(EXEC_PLUGINS) $(NET_PLUGINS) $(NETSTAT_PLUGINS) $(HTTP_PLUGINS)
IMAGE_NAME := qcontrol-sdk-rust-demo

.PHONY: all setup-sdk build build-file build-exec build-net build-netstat build-http clean docker-build dev

all: build

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

build: build-file build-exec build-net build-netstat build-http

build-file:
	@for p in $(FILE_PLUGINS); do $(MAKE) -C $$p build; done

build-exec:
	@for p in $(EXEC_PLUGINS); do $(MAKE) -C $$p build; done

build-net:
	@for p in $(NET_PLUGINS); do $(MAKE) -C $$p build; done

build-netstat:
	@for p in $(NETSTAT_PLUGINS); do $(MAKE) -C $$p build; done

build-http:
	@for p in $(HTTP_PLUGINS); do $(MAKE) -C $$p build; done

clean:
	cargo clean
	rm -rf .cargo
	@for p in $(PLUGINS); do $(MAKE) -C $$p clean; done

docker-build:
	docker build -t $(IMAGE_NAME) -f Dockerfile ..

dev: docker-build
	docker run --rm -it $(IMAGE_NAME)
