[private]
@default:
	just --list

# Build for prod using cross
build-release:
	# disabling sccache and clang linker
	cross b --release \
		--features=prod \
		--target=x86_64-unknown-linux-musl \
		--config build.rustc-wrapper="''" \
		--config target.x86_64-unknown-linux-gnu.linker="'gcc'"

# Run tests
test what='--workspace':
	cargo test {{what}}

deploy ssh-path:
	scp target/x86_64-unknown-linux-musl/release/app-pulse-bot {{ssh-path}}

# Add new migration
add-migrate name:
	sqlx migrate add '{{ name }}'

# Bump version in Cargo.toml
bump to:
	@# https://github.com/ceejbot/tomato
	tomato set 'workspace.package.version' {{to}} Cargo.toml
	cargo c
