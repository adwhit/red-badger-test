# Red Badger Test

For fun, I tried to do it with zero dependencies. Usually for something with a CLI I would pull in `clap` and for parsing I would use something like `regex` or `nom`. But this test was simple enough to do it by hand.

## Usage

You will need the Rust/Cargo toolchain, installable from [rustup.rs](http://rustup.rs).

Run:

```rust
cargo run --release -- <PATH/TO/INPUT>
```

Test:

```rust
cargo test
```

Also try:
```rust
cargo run --release -- test-input
```
