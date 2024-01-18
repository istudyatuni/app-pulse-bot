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

# Bump package and bot versions in Cargo.toml
bump package bot:
	@# https://github.com/ceejbot/tomato
	tomato set 'workspace.package.version' {{package}} Cargo.toml
	tomato set 'package.metadata.bot.version' {{bot}} Cargo.toml
	cargo c
