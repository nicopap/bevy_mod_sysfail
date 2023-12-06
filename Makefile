CLIPPY_ARGS=-- -D clippy::all -D clippy::pedantic -D clippy::nursery \
	-D missing_docs \
	-D clippy::undocumented_unsafe_blocks \
	-W clippy::needless-pass-by-value \
	-A clippy::missing_const_for_fn \
	-A clippy::module_name_repetitions \
	-A clippy::redundant_pub_crate

check:
	cargo check --examples

run:
	cargo run --example all_attributes

pre-hook:
	RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
	cargo clippy --workspace $(CLIPPY_ARGS)
	cargo fmt --all -- --check
	cargo clippy --workspace --no-default-features
	cargo test -j12
