check:
	SKIP_GUEST_BUILD=1 cargo check
	SKIP_GUEST_BUILD=1 cargo check --manifest-path crates/provers/risc0/guest-celestia/Cargo.toml
	SKIP_GUEST_BUILD=1 cargo check --manifest-path crates/provers/risc0/guest-mock/Cargo.toml
	SKIP_GUEST_BUILD=1 cargo check --manifest-path crates/provers/sp1/guest-celestia/Cargo.toml
	SKIP_GUEST_BUILD=1 cargo check --manifest-path crates/provers/sp1/guest-mock/Cargo.toml

lint:
	SKIP_GUEST_BUILD=1 cargo fmt --all -- --check
	SKIP_GUEST_BUILD=1 cargo check
	SKIP_GUEST_BUILD=1 cargo check --features celestia_da --features risc0 --no-default-features
	SKIP_GUEST_BUILD=1 cargo clippy
	SKIP_GUEST_BUILD=1 cargo clippy --features celestia_da --features risc0 --no-default-features
	SKIP_GUEST_BUILD=1 cargo clippy --features celestia_da --features sp1 --no-default-features
	zepter
	zepter
	zepter


install-risc0-toolchain:
	curl -L https://risczero.com/install | bash
	~/.risc0/bin/rzup install cargo-risczero v1.1.2
	cargo risczero install --version r0.1.79.0
	@echo "Risc0 toolchain version:"
	cargo +risc0 --version


install-sp1-toolchain:
	@echo "TOKEN"
	@echo "$$GITHUB_TOKEN" 
	curl -L https://raw.githubusercontent.com/succinctlabs/sp1/main/sp1up/install | bash
	~/.sp1/bin/sp1up --token "$$GITHUB_TOKEN"
	~/.sp1/bin/cargo-prove prove --version
	~/.sp1/bin/cargo-prove prove install-toolchain --token "$$GITHUB_TOKEN"
	@echo "SP1 toolchain version:"
	cargo +succinct --version

clean:
	@cargo clean
	@cargo clean --manifest-path crates/provers/risc0/guest-celestia/Cargo.toml
	@cargo clean --manifest-path crates/provers/risc0/guest-mock/Cargo.toml
	@cargo clean --manifest-path crates/sp1/risc0/guest-celestia/Cargo.toml
	@cargo clean --manifest-path crates/sp1/risc0/guest-mock/Cargo.toml
	rm -rf rollup-starter-data/
	rm -rf crates/rollup/mock_da.sqlite
