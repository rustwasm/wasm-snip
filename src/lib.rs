/*!

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

## Executable

To install the `wasm-snip` executable, run

```text
$ cargo install wasm-snip
```

You can use `wasm-snip` to remove the `annoying_space_waster`
function from `input.wasm` and put the new binary in `output.wasm` like this:

```text
$ wasm-snip input.wasm -o output.wasm annoying_space_waster
```

For information on using the `wasm-snip` executable, run

```text
$ wasm-snip --help
Replace a wasm function with an `unreachable`.

USAGE:
wasm-snip [FLAGS] [OPTIONS] <input> [--] [function]...

FLAGS:
-h, --help                        Prints help information
--snip-rust-fmt-code          Snip Rust's `std::fmt` and `core::fmt` code.
--snip-rust-panicking-code    Snip Rust's `std::panicking` and `core::panicking` code.
-V, --version                     Prints version information

OPTIONS:
-o, --output <output>         The path to write the output wasm file to. Defaults to stdout.
-p, --pattern <pattern>...    Snip any function that matches the given regular expression.

ARGS:
<input>          The input wasm file containing the function(s) to snip.
<function>...    The specific function(s) to snip. These must match exactly. Use the -p flag for fuzzy matching.
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
extern crate regex;

use failure::ResultExt;
use parity_wasm::elements;
use std::collections::HashMap;
use std::path;

/// Options for controlling which functions in what `.wasm` file should be
/// snipped.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The input `.wasm` file that should have its functions snipped.
    pub input: path::PathBuf,

    /// The functions that should be snipped from the `.wasm` file.
    pub functions: Vec<String>,

    /// The regex patterns whose matches should be snipped from the `.wasm`
    /// file.
    pub patterns: Vec<String>,

    /// Should Rust `std::fmt` and `core::fmt` functions be snipped?
    pub snip_rust_fmt_code: bool,

    /// Should Rust `std::panicking` and `core::panicking` functions be snipped?
    pub snip_rust_panicking_code: bool,
}

/// Snip the functions from the input file described by the options.
pub fn snip(options: Options) -> Result<elements::Module, failure::Error> {
    let mut options = options;

    let mut module = elements::deserialize_file(&options.input)?
        .parse_names()
        .unwrap();

    let names: HashMap<String, usize> = module
        .names_section_names()
        .ok_or(failure::err_msg(
            "missing \"name\" section; did you build with debug symbols?",
        ))?
        .into_iter()
        .map(|(index, name)| (name, index as usize))
        .collect();

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

        // Snip the exact match functions.
        for to_snip in &options.functions {
            let idx = names.get(to_snip).ok_or(format_err!(
                "asked to snip '{}', but it isn't present",
                to_snip
            ))?;

            snip_nth(*idx, num_imports, code)
                .with_context(|_| format!("when attempting to snip '{}'", to_snip))?;
        }

        // Snip the Rust `fmt` code, if requested.
        if options.snip_rust_fmt_code {
            // Mangled symbols.
            options.patterns.push(".*4core3fmt.*".into());
            options.patterns.push(".*3std3fmt.*".into());

            // Mangled in impl.
            options.patterns.push(r#".*core\.\.fmt\.\..*"#.into());
            options.patterns.push(r#".*std\.\.fmt\.\..*"#.into());

            // Demangled symbols.
            options.patterns.push(".*core::fmt::.*".into());
            options.patterns.push(".*std::fmt::.*".into());
        }

        // Snip the Rust `panicking` code, if requested.
        if options.snip_rust_panicking_code {
            // Mangled symbols.
            options.patterns.push(".*4core9panicking.*".into());
            options.patterns.push(".*3std9panicking.*".into());

            // Mangled in impl.
            options.patterns.push(r#".*core\.\.panicking\.\..*"#.into());
            options.patterns.push(r#".*std\.\.panicking\.\..*"#.into());

            // Demangled symbols.
            options.patterns.push(".*core::panicking::.*".into());
            options.patterns.push(".*std::panicking::.*".into());
        }

        let re_set = regex::RegexSet::new(options.patterns)?;

        for (name, idx) in names {
            if idx >= num_imports && re_set.is_match(&name) {
                snip_nth(idx, num_imports, code)?;
            }
        }
    }

    Ok(module)
}

fn snip_nth(
    n: usize,
    num_imports: usize,
    code: &mut elements::CodeSection,
) -> Result<(), failure::Error> {
    if n < num_imports {
        bail!("cannot snip imported functions");
    }

    let body = code.bodies_mut()
        .get_mut(n - num_imports)
        .ok_or(failure::err_msg(format!(
            "index {} is out of bounds of the code section",
            n - num_imports
        )))?;

    *body.code_mut().elements_mut() = vec![elements::Opcode::Unreachable, elements::Opcode::End];

    Ok(())
}

trait NamesSectionNames {
    fn names_section_names(&self) -> Option<elements::NameMap>;
}

impl NamesSectionNames for elements::Module {
    fn names_section_names(&self) -> Option<elements::NameMap> {
        for section in self.sections() {
            if let &elements::Section::Name(elements::NameSection::Function(ref name_section)) =
                section
            {
                return Some(name_section.names().clone());
            }
        }

        None
    }
}
