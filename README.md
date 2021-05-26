# ed (encode/decode)

*A minimalist crate for deterministic binary encodings in Rust.*

[![codecov](https://codecov.io/gh/nomic-io/ed/branch/master/graph/badge.svg?token=BZK1DP4CF4)](https://codecov.io/gh/nomic-io/ed)
[![Crate](https://img.shields.io/crates/v/ed.svg)](https://crates.io/crates/ed)
[![API](https://docs.rs/ed/badge.svg)](https://docs.rs/ed)

## Overview

This crate provides `Encode` and `Decode` traits which can be implemented for any type that can be converted to or from bytes, and implements these traits for many built-in Rust types. It also provides derive macros so that `Encode` and `Decode` can be easily derived for structs.

`ed` is far simpler than `serde` because it does not attempt to create an abstraction which allows arbitrary kinds of encoding (JSON, MessagePack, etc.), and instead forces focuses on binary encodings. It is also significantly faster than [`bincode`](https://docs.rs/bincode), the leading binary `serde` serializer.

One aim of `ed` is to force top-level type authors to design their own encoding, rather than attempting to provide a one-size-fits-all encoding scheme. This lets users of `ed` be sure their encodings are as effiient as possible, and makes it easier to understand the encoding for compatability in other languages or libraries (contrasted with something like `bincode`, where it is not obvious how a type is being encoded without understanding the internals of `bincode`). 

Another property of this crate is a focus on determinism (important for cryptographically hashed types) - built-in encodings are always big-endian and there are no provided encodings for floating point numbers or `usize`.

## Usage 
```rust
use ed::{Encode, Decode};

// traits are implemented for built-in types
let bytes = 123u32.encode()?; // `bytes` is a Vec<u8>
let n = u32::decode(bytes.as_slice())?; // `n` is a u32

// derive macros are available
#[derive(Encode, Decode)]
struct Foo {
  bar: (u32, u32),
  baz: Vec<u8>
}

// encoding and decoding can be done in-place to reduce allocations
let mut bytes = vec![0xba; 40];
let mut foo = Foo {
  bar: (0, 0),
  baz: Vec::with_capacity(32)
};

// in-place decode, re-using pre-allocated `foo.baz` vec
foo.decode_into(bytes.as_slice())?;
assert_eq!(foo, Foo {
  bar: (0xbabababa, 0xbabababa),
  baz: vec![0xba; 32]
});

// in-place encode, into pre-allocated `bytes` vec
bytes.clear();
foo.encode_into(&mut bytes)?;
```
