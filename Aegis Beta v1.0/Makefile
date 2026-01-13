.PHONY: all build run setup clean install test

all: build

build:
	cd backend && cargo build --release

run: build
	cd backend && cargo run

setup:
	chmod +x scripts/setup.sh
	sudo scripts/setup.sh

install-deps:
	chmod +x scripts/install-deps.sh
	scripts/install-deps.sh

clean:
	cd backend && cargo clean
	rm -rf data/*

test:
	cd backend && cargo test

install: build
	sudo cp backend/target/release/vm-manager /usr/local/bin/
	sudo mkdir -p /opt/vm-manager
	sudo cp -r frontend/* /opt/vm-manager/frontend/
	sudo cp config/default.toml /etc/vm-manager.toml

uninstall:
	sudo systemctl stop vm-manager || true
	sudo systemctl disable vm-manager || true
	sudo rm -f /usr/local/bin/vm-manager
	sudo rm -rf /opt/vm-manager
	sudo rm -f /etc/systemd/system/vm-manager.service
	sudo rm -f /etc/vm-manager.toml