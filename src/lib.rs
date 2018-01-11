/*!

[![](https://docs.rs/wasm-snip/badge.svg)](https://docs.rs/wasm-snip/) [![](https://img.shields.io/crates/v/wasm-snip.svg)](https://crates.io/crates/wasm-snip) [![](https://img.shields.io/crates/d/wasm-snip.png)](https://crates.io/crates/wasm-snip) [![Build Status](https://travis-ci.org/fitzgen/wasm-snip.png?branch=master)](https://travis-ci.org/fitzgen/wasm-snip)

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

## Executable

To install the `wasm-snip` executable, run

```text
$ cargo install wasm-snip
```

For information on using the `wasm-snip` executable, run

```text
$ wasm-snip --help
```

## Library

To use `wasm-snip` as a library, add this to your `Cargo.toml`:

```toml
[dependencies.wasm-snip]
# Do not build the executable.
default-features = false
```

See [docs.rs/wasm-snip][docs] for API documentation.

[docs]: https://docs.rs/wasm-snip

## License

Licensed under either of

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)

 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

## Contributing

See
[CONTRIBUTING.md](https://github.com/fitzgen/wasm-snip/blob/master/CONTRIBUTING.md)
for hacking.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

 */

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

#[macro_use]
extern crate failure;
extern crate parity_wasm;

use parity_wasm::elements::{self, Deserialize};
use std::collections::hash_map::{Entry, HashMap};
use std::path;

/// Options for controlling which functions in what `.wasm` file should be
/// snipped.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The input `.wasm` file that should have its functions snipped.
    pub input: path::PathBuf,

    /// The functions that should be snipped from the `.wasm` file.
    pub functions: Vec<String>,
}

const FUNCTION_NAMES: u8 = 1;

// Adapted from `wasm-gc`; waiting for the name section support to be upstreamed
// into parity-wasm.
fn decode_name_map<'a>(mut bytes: &'a [u8]) -> Result<HashMap<String, usize>, failure::Error> {
    while !bytes.is_empty() {
        let name_type = u8::from(elements::VarUint7::deserialize(&mut bytes)?);
        let name_payload_len = u32::from(elements::VarUint32::deserialize(&mut bytes)?);
        let (these_bytes, rest) = bytes.split_at(name_payload_len as usize);

        if name_type == FUNCTION_NAMES {
            bytes = these_bytes;
        } else {
            bytes = rest;
            continue;
        }

        let count = u32::from(elements::VarUint32::deserialize(&mut bytes)?);
        let mut names = HashMap::with_capacity(count as usize);
        for _ in 0..count {
            let index = usize::from(elements::VarUint32::deserialize(&mut bytes)?);

            let name_len = usize::from(elements::VarUint32::deserialize(&mut bytes)?);
            let (name, rest) = bytes.split_at(name_len);
            let name = String::from_utf8(name.to_vec())?;

            match names.entry(name) {
                Entry::Occupied(entry) => {
                    bail!(format!("duplicate name entries for '{}'", entry.key()));
                }
                Entry::Vacant(entry) => {
                    entry.insert(index);
                }
            }

            bytes = rest;
        }
        return Ok(names);
    }

    return Ok(Default::default());
}

/// Snip the functions from the input file described by the options.
pub fn snip(options: Options) -> Result<elements::Module, failure::Error> {
    let mut module = elements::deserialize_file(&options.input)?;

    let names = module
        .sections()
        .iter()
        .filter_map(|section| match *section {
            elements::Section::Custom(ref custom) if custom.name() == "name" => Some(custom),
            _ => None,
        })
        .next()
        .and_then(|name_section| decode_name_map(name_section.payload()).ok())
        .ok_or(failure::err_msg("missing \"name\" section"))?;

    {
        let num_imports = module
            .import_section()
            .map_or(0, |imports| imports.entries().len());

        let code = module
            .sections_mut()
            .iter_mut()
            .filter_map(|section| match *section {
                elements::Section::Code(ref mut code) => Some(code),
                _ => None,
            })
            .next()
            .ok_or(failure::err_msg("missing code section"))?;

        for function in &options.functions {
            let idx = names.get(function).ok_or(failure::err_msg(format!(
                "'{}' is not in the name section",
                function
            )))?;

            let mut body = code.bodies_mut()
                .get_mut(*idx - num_imports)
                .ok_or(failure::err_msg(format!(
                    "index for '{}' is out of bounds of the code section",
                    function
                )))?;

            *body.code_mut().elements_mut() =
                vec![elements::Opcode::Unreachable, elements::Opcode::End];
        }
    }

    Ok(module)
}
