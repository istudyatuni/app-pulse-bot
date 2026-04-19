rust-version := "1.95.0"

[private]
@default:
	just --list

# Run checks
check:
	cargo fmt --check
	cargo clippy -- -D warnings
	@just test

# Run tests
test what='--workspace':
	cargo nextest run {{what}}

build-deploy ssh ssh-path post-deploy-ssh-script: check
	@just build-release
	@just deploy '{{ssh}}:{{ssh-path}}'
	ssh '{{ssh}}' '{{post-deploy-ssh-script}}'

# Add new migration
add-migrate name:
	sqlx migrate add '{{ name }}'

# Bump package and bot versions in Cargo.toml
bump package bot:
	@# https://github.com/ceejbot/tomato
	tomato set 'workspace.package.version' {{package}} Cargo.toml
	tomato set 'package.metadata.bot.version' {{bot}} Cargo.toml
	cargo c

# enter shell with dependencies
shell:
    nix develop --profile flake.drv ".#"

# Build for prod using cross
[private]
build-release:
	@# disabling sccache and clang linker with --config
	@# CARGO_HOME and /tmp/.cargo is used to use local cargo download cache
	docker run --rm \
		-v "$(pwd)":/build \
		-v "$HOME/.cargo":/tmp/.cargo \
		-w /build \
		--env=CARGO_HOME=/tmp/.cargo \
		"clux/muslrust:{{ rust-version }}-stable" \
		cargo build --release \
			--features=prod \
			--target=x86_64-unknown-linux-musl \
			--config build.rustc-wrapper="''" \
			--config target.x86_64-unknown-linux-gnu.linker="'gcc'"

[private]
deploy ssh-path:
	scp target/x86_64-unknown-linux-musl/release/app-pulse-bot '{{ssh-path}}'
