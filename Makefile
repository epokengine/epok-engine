# Common development commands. Recipes use Cargo directly, without POSIX shell
# utilities, so GNU Make can invoke them through cmd.exe or a Unix shell.
.DEFAULT_GOAL := help

POWERSHELL ?= powershell
UNAME_S := $(shell uname -s)
# Windows installs the interpreter as python; the Unix hosts we support ship
# python3 and may not provide an unversioned alias at all.
ifeq ($(OS),Windows_NT)
PYTHON ?= python
else
PYTHON ?= python3
endif
ifeq ($(UNAME_S),Darwin)
CARGO ?= ./tools/cargo-macos.sh
else
CARGO ?= cargo
endif
PROJECT ?=
ARGS ?=

PROJECT_ARG = $(if $(strip $(PROJECT)),--project "$(PROJECT)")
GAME_PROJECT = $(if $(strip $(PROJECT)),$(PROJECT),examples/sample-game)

.PHONY: help run run-release build release app check test lint fmt fmt-check setup setup-repair build-psx build-state reset-build-state api-docs api-docs-check gameplay-api-inventory gameplay-api-check

help:
	@echo Epok development commands
	@echo   make run          - Open the editor, optionally set PROJECT and ARGS
	@echo   make run-release  - Open the optimized editor
	@echo   make build        - Compile the debug editor
	@echo   make release      - Compile the optimized editor binary, not a distribution
	@echo   make app          - On macOS, create Epok Engine.app and optionally copy it to Desktop
	@echo   make check        - Check formatting, tests, lint and debug build in order
	@echo   make test         - Run GPU-free default tests
	@echo   make lint         - Run Clippy with warnings as errors
	@echo   make fmt          - Format Rust source
	@echo   make fmt-check    - Check Rust formatting without changing files
	@echo   make setup        - Install or verify PSX tools for the current supported host
	@echo   make setup-repair - Restore PSX tool files from verified archives
	@echo   make build-psx    - Build PROJECT for PSX, defaults to examples/sample-game
	@echo   make build-state  - Show the local version and successful build count
	@echo   make reset-build-state - Reset the local build counter
	@echo   make api-docs      - Regenerate the Epok and PsyQo C++ API reference
	@echo   make api-docs-check - Verify the checked-in API reference is current
	@echo   make gameplay-api-inventory - Verify the classified gameplay API denominator
	@echo   make gameplay-api-check - Require complete C++/Blueprint/Lua gameplay parity

run: build
	$(CARGO) run --locked -- $(PROJECT_ARG) $(ARGS)

run-release: release
	$(CARGO) run --locked --release -- $(PROJECT_ARG) $(ARGS)

build:
	$(PYTHON) tools/build_state.py record --base develop -- $(CARGO) build --locked

release:
	$(PYTHON) tools/build_state.py record --base develop -- $(CARGO) build --locked --release

# A macOS .app is one item in Finder, while retaining the native executable and
# its companion extractor in Contents/MacOS where the editor expects them.
app:
	./tools/package-macos-app.sh

# Keep these sequential even with make -j: Cargo commands share build outputs.
# Make stops immediately if any command fails.
check:
	$(CARGO) fmt --all -- --check
	$(CARGO) build --locked --bin epok-header-tool
	$(CARGO) test --locked
	$(CARGO) clippy --locked --all-targets -- -D warnings
	$(PYTHON) tools/build_state.py record --base develop -- $(CARGO) build --locked

test:
	$(CARGO) build --locked --bin epok-header-tool
	$(CARGO) test --locked

lint:
	$(CARGO) clippy --locked --all-targets -- -D warnings

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

build-psx:
	$(PYTHON) tools/build_state.py record --base develop -- $(CARGO) run --locked -- --project "$(GAME_PROJECT)" --build-psx

build-state:
	$(PYTHON) tools/build_state.py show

reset-build-state:
	$(PYTHON) tools/build_state.py reset

api-docs:
	$(PYTHON) tools/generate-api-reference.py

api-docs-check:
	$(PYTHON) tools/generate-api-reference.py --check

gameplay-api-inventory:
	$(PYTHON) tools/gameplay_api_parity.py --inventory-only

gameplay-api-check:
	$(PYTHON) tools/gameplay_api_parity.py

# Only dependency provisioning is platform-specific here. Add platform setup
# implementations when their SDK/tool distributions and editor support exist.
ifeq ($(OS),Windows_NT)
setup:
	$(POWERSHELL) -NoProfile -ExecutionPolicy Bypass -File tools/setup.ps1

setup-repair:
	$(POWERSHELL) -NoProfile -ExecutionPolicy Bypass -File tools/setup.ps1 -Repair
else
ifeq ($(UNAME_S),Darwin)
setup:
	./tools/setup-macos.sh

setup-repair:
	./tools/setup-macos.sh --repair
else
ifeq ($(UNAME_S),Linux)
setup:
	./tools/setup-linux.sh

setup-repair:
	./tools/setup-linux.sh --repair
else
setup setup-repair:
	$(error PSX dependency setup currently supports Windows, macOS and Linux x86_64)
endif
endif
endif
