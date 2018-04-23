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

extern crate failure;
extern crate parity_wasm;

use parity_wasm::elements;
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

/// Snip the functions from the input file described by the options.
pub fn snip(options: Options) -> Result<elements::Module, failure::Error> {
    let mut module = elements::deserialize_file(&options.input)?.parse_names().unwrap();

    let names = module
        .names_section_names()
        .ok_or(failure::err_msg("missing \"name\" section"))?;

    {
        let num_imports = module.import_count(elements::ImportCountType::Function);

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
            let (idx, _) = names
                .iter()
                .find(|&(_, name)| name == function)
                .unwrap();

            let mut body = code.bodies_mut()
                .get_mut(idx as usize - num_imports)
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

trait NamesSectionNames {
    fn names_section_names(&self) -> Option<elements::NameMap>;
}

impl NamesSectionNames for elements::Module {
    fn names_section_names(&self) -> Option<elements::NameMap> {
        for section in self.sections() {
            if let &elements::Section::Name(elements::NameSection::Function(ref name_section)) = section {
                return Some(name_section.names().clone());
            }
        }

        None
    }
}
