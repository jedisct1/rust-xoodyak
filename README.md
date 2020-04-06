![GitHub CI](https://github.com/jedisct1/rust-xoodyak/workflows/Rust/badge.svg)

# Xoodyak for Rust

This is a Rust implementation of [Xoodyak](https://csrc.nist.gov/CSRC/media/Projects/lightweight-cryptography/documents/round-2/spec-doc-rnd2/Xoodyak-spec-round2.pdf), a cryptographic primitive that can be used for hashing, encryption, MAC computation and authenticated encryption.

* `no_std`-friendly
* Lightweight
* Can be compiled to WebAssembly/WASI
* Session support
* Safe Rust interface
* AEAD with attached and detached tags
* In-place encryption
* Ratcheting
* Variable-length output hashing, authentication
* `squeeze_more()`, `absorb_more()` for streaming.

# [API documentation](https://docs.rs/xoodyak)
