<div align="center">

  <h1><code>wasm-snip</code></h1>

  <strong><code>wasm-snip</code> replaces a Wasm function's body with an <code>unreachable</code> instruction.</strong>

  <p>
    <a href="https://travis-ci.org/rustwasm/wasm-snip"><img src="https://img.shields.io/travis/rustwasm/wasm-snip.svg?style=flat-square" alt="Build Status" /></a>
    <a href="https://crates.io/crates/wasm-snip"><img src="https://img.shields.io/crates/v/wasm-snip.svg?style=flat-square" alt="Crates.io version" /></a>
    <a href="https://crates.io/crates/wasm-snip"><img src="https://img.shields.io/crates/d/wasm-snip.svg?style=flat-square" alt="Download" /></a>
    <a href="https://docs.rs/wasm-snip"><img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
  </p>

  <h3>
    <a href="https://docs.rs/wasm-snip">API Docs</a>
    <span> | </span>
    <a href="https://github.com/rustwasm/wasm-snip/blob/master/CONTRIBUTING.md">Contributing</a>
    <span> | </span>
    <a href="https://discordapp.com/channels/442252698964721669/443151097398296587">Chat</a>
  </h3>

  <sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">The Rust and WebAssembly Working Group</a></sub>
</div>

## About


`wasm-snip` replaces a WebAssembly function's body with an `unreachable`.

Maybe you know that some function will never be called at runtime, but the
compiler can't prove that at compile time? Snip it! All the functions it
transitively called &mdash; which weren't called by anything else and therefore
could also never be called at runtime &mdash; will get removed too.

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

You can use `wasm-snip` to remove the `annoying_space_waster`
function from `input.wasm` and put the new binary in `output.wasm` like this:

```
$ wasm-snip input.wasm -o output.wasm annoying_space_waster
```

For information on using the `wasm-snip` executable, run

```
$ wasm-snip --help
```

And you'll get the most up-to-date help text, like:

```
Replace a wasm function with an `unreachable`.

USAGE:
wasm-snip [FLAGS] [OPTIONS] <input> [--] [function]...

FLAGS:
-h, --help                    Prints help information
--snip-rust-fmt-code          Snip Rust's `std::fmt` and `core::fmt` code.
--snip-rust-panicking-code    Snip Rust's `std::panicking` and `core::panicking` code.
-V, --version                 Prints version information

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
[CONTRIBUTING.md](https://github.com/rustwasm/wasm-snip/blob/master/CONTRIBUTING.md)
for hacking.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

