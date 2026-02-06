# Originally from the project dannyhammer/toad on GitHub.
# See: https://github.com/dannyhammer/toad/blob/main/Makefile
#
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.


# If on Windows, add the .exe extension to the executable and use PowerShell instead of `sed`
ifeq ($(OS),Windows_NT)
	EXT := .exe
	NAME := $(shell powershell -Command "(Get-Content Cargo.toml | Select-String '^name =').Line -replace '.*= ', '' -replace '\"', ''")
	VERSION := $(shell powershell -Command "(Get-Content Cargo.toml | Select-String '^version =').Line -replace '.*= ', '' -replace '\"', ''")
else
	EXT :=
	NAME := $(shell sed -n '0,/^name = "\(.*\)"/s//\1/p' Cargo.toml)
	VERSION := $(shell sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml)
endif

# OpenBench specifies that the binary name should be changeable with the EXE parameter
ifndef EXE
	EXE := $(NAME)-$(VERSION)$(EXT)
else
	EXE := $(EXE)$(EXT)
endif

# Force clang to be our CC compiler when not on Windows
ifneq ($(OS),Windows_NT)
	export HOST_CC=clang
endif


# Compile an executable for use with OpenBench
openbench:
	@echo Compiling $(EXE) for OpenBench
	cargo rustc --release -p byte-knight --bin byte-knight -- -C target-cpu=native --emit link=$(EXE)

# Remove the EXE created
clean:
	@echo Removing $(EXE)
	rm $(EXE)
