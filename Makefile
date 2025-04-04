# Constructs targets for the current CPU architecture from the configuration
# files for each package, `packages/*.cfg`.

# Determine the CPU architecture. Corresponds to the architecture names uses
# by Apt packages.
arch := $(shell dpkg --print-architecture)
dest := build
include /etc/lsb-release

all: tembox packages

# Build tarballs for all packages from `packages/*.cfg`.
packages: $(patsubst packages/%.cfg,$(dest)/$(DISTRIB_CODENAME)/tembo-%_$(arch).tgz,$(wildcard packages/*.cfg))
$(dest)/jammy/tembo-%_$(arch).tgz:
	@./bin/repackage $* $(dest)
$(dest)/noble/tembo-%_$(arch).tgz:
	@./bin/repackage $* $(dest)

# Install tarballs for all packages from `packages/*.cfg`.
install: $(patsubst packages/%.cfg,install-%,$(wildcard packages/*.cfg))
install-%: $(dest)/*/tembo-%_$(arch).tgz
	./bin/install_package $* $(dest)

tembox: target/release/tembox

target/release/tembox: Cargo.toml src/main.rs
	cargo build --release

clean:
	@rm -rf $(dest) target
