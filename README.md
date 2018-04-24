# `wasm-snip`

[![](https://docs.rs/wasm-snip/badge.svg)](https://docs.rs/wasm-snip/)
[![](https://img.shields.io/crates/v/wasm-snip.svg)](https://crates.io/crates/wasm-snip)
[![](https://img.shields.io/crates/d/wasm-snip.png)](https://crates.io/crates/wasm-snip)
[![Build Status](https://travis-ci.org/fitzgen/wasm-snip.png?branch=master)](https://travis-ci.org/fitzgen/wasm-snip)

`wasm-snip` replaces a WebAssembly function's body with an `unreachable`.

Maybe you know that some function will never be called at runtime, but the
compiler can't prove that at compile time? Snip it! Then run
[`wasm-gc`][wasm-gc] again and all the functions it transitively called (which
could also never be called at runtime) will get removed too.

[wasm-gc]: https://github.com/alexcrichton/wasm-gc

Very helpful when shrinking the size of WebAssembly binaries!

This functionality relies on the "name" section being present in the `.wasm`
file, so build with debug symbols:

```toml
[profile.release]
debug = true
```

* [Executable](#executable)
* [Library](#library)
* [License](#license)
* [Contributing](#contributing)

### Executable

To install the `wasm-snip` executable, run

```
$ cargo install wasm-snip
```

For information on using the `wasm-snip` executable, run

```
$ wasm-snip --help
Replace a wasm function with an `unreachable`.

USAGE:
wasm-snip [OPTIONS] <input> [--] [function]...

FLAGS:
-h, --help                  Prints help information
    --snip-rust-fmt-code    Snip Rust's `std::fmt` and `core::fmt` code.
-V, --version               Prints version information

OPTIONS:
-o, --output <output>         The path to write the output wasm file to. Defaults to stdout.
-p, --pattern <pattern>...    Snip any function that matches the given regular expression.

ARGS:
<input>          The input wasm file containing the function(s) to snip.
<function>...    The specific function(s) to snip. These must match exactly. Use the -p flag for fuzzy matching.
```

### Library

To use `wasm-snip` as a library, add this to your `Cargo.toml`:

```toml
[dependencies.wasm-snip]
# Do not build the executable.
default-features = false
```

See [docs.rs/wasm-snip][docs] for API documentation.

[docs]: https://docs.rs/wasm-snip

### License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contributing

See
[CONTRIBUTING.md](https://github.com/fitzgen/wasm-snip/blob/master/CONTRIBUTING.md)
for hacking.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

