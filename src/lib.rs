/*!

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
```

And you'll get the most up-to-date help text, like:

```text
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
[CONTRIBUTING.md](https://github.com/rustwasm/wasm-snip/blob/master/CONTRIBUTING.md)
for hacking.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

 */

#![deny(missing_docs)]
#![deny(missing_debug_implementations)]

use failure::ResultExt;
use rayon::prelude::*;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path;
use walrus::ir::VisitorMut;

/// Input configuration.
#[derive(Clone, Debug)]
pub enum Input {
    /// The input `.wasm` file that should have its function snipped.
    File(path::PathBuf),
    /// The input WebAssembly blob that should have its function snipped.
    Buffer(Vec<u8>),
    // TODO: Support Walrus module directly.
    // Module(walrus::Module),
}

impl Default for Input {
    fn default() -> Self {
        Input::File(path::PathBuf::default())
    }
}

/// Options for controlling which functions in what `.wasm` file should be
/// snipped.
#[derive(Clone, Debug, Default)]
pub struct Options {
    /// The functions that should be snipped from the `.wasm` file.
    pub functions: Vec<String>,

    /// The regex patterns whose matches should be snipped from the `.wasm`
    /// file.
    pub patterns: Vec<String>,

    /// The exports kept while others are snipped from the `.wasm`
    pub kept_exports: Vec<String>,

    /// The regex patterns of exports kept while others are snipped from the `.wasm`.
    pub kept_export_patterns: Vec<String>,

    /// Should Rust `std::fmt` and `core::fmt` functions be snipped?
    pub snip_rust_fmt_code: bool,

    /// Should Rust `std::panicking` and `core::panicking` functions be snipped?
    pub snip_rust_panicking_code: bool,

    /// Should we skip generating [the "producers" custom
    /// section](https://github.com/WebAssembly/tool-conventions/blob/master/ProducersSection.md)?
    pub skip_producers_section: bool,
}

/// Snip the functions from the input file described by the options.
pub fn snip(module: &mut walrus::Module, options: Options) -> Result<(), failure::Error> {
    if !options.skip_producers_section {
        module
            .producers
            .add_processed_by("wasm-snip", env!("CARGO_PKG_VERSION"));
    }

    let re_kept_set = regex::RegexSet::new(&options.kept_export_patterns)?;
    let export_names: HashSet<String> = options.kept_exports.iter().cloned().collect();
    let exports_to_delete = find_exports_to_delete(&module, &export_names, &re_kept_set);

    let names: HashSet<String> = options.functions.iter().cloned().collect();
    let re_set = build_regex_set(options).context("failed to compile regex")?;
    let to_snip = find_functions_to_snip(&module, &names, &re_set);

    delete_exports(module, &exports_to_delete);
    replace_calls_with_unreachable(module, &to_snip);
    unexport_snipped_functions(module, &to_snip);
    unimport_snipped_functions(module, &to_snip);
    snip_table_elements(module, &to_snip);
    delete_functions_to_snip(module, &to_snip);
    walrus::passes::gc::run(module);

    Ok(())
}

fn build_regex_set(mut options: Options) -> Result<regex::RegexSet, failure::Error> {
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

    Ok(regex::RegexSet::new(options.patterns)?)
}

fn find_functions_to_snip(
    module: &walrus::Module,
    names: &HashSet<String>,
    re_set: &regex::RegexSet,
) -> HashSet<walrus::FunctionId> {
    module
        .funcs
        .par_iter()
        .filter_map(|f| {
            f.name.as_ref().and_then(|name| {
                if names.contains(name) || re_set.is_match(name) {
                    Some(f.id())
                } else {
                    None
                }
            })
        })
        .collect()
}

fn is_function(export: &walrus::Export) -> Option<&walrus::Export> {
    match export.item {
        walrus::ExportItem::Function(_) => Some(export),
        _ => None,
    }
}
fn find_exports_to_delete(
    module: &walrus::Module,
    names: &HashSet<String>,
    re_set: &regex::RegexSet,
) -> HashSet<walrus::ExportId> {
    module
        .exports
        .iter()
        .filter_map(is_function)
        .filter_map(|e| {
            if names.contains(&e.name) || re_set.is_match(&e.name) {
                None
            } else {
                Some(e.id())
            }
        })
        .collect()
}

fn delete_exports(module: &mut walrus::Module, to_snip: &HashSet<walrus::ExportId>) {
    for id in to_snip.iter().cloned() {
        module.exports.delete(id)
    }
}

fn delete_functions_to_snip(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    for f in to_snip.iter().cloned() {
        module.funcs.delete(f);
    }
}

fn replace_calls_with_unreachable(
    module: &mut walrus::Module,
    to_snip: &HashSet<walrus::FunctionId>,
) {
    struct Replacer<'a> {
        to_snip: &'a HashSet<walrus::FunctionId>,
    }

    impl Replacer<'_> {
        fn should_snip_call(&self, instr: &walrus::ir::Instr) -> bool {
            if let walrus::ir::Instr::Call(walrus::ir::Call { func }) = instr {
                if self.to_snip.contains(func) {
                    return true;
                }
            }
            false
        }
    }

    impl VisitorMut for Replacer<'_> {
        fn visit_instr_mut(&mut self, instr: &mut walrus::ir::Instr) {
            if self.should_snip_call(instr) {
                *instr = walrus::ir::Unreachable {}.into();
            }
        }
    }

    module.funcs.par_iter_local_mut().for_each(|(id, func)| {
        // Don't bother transforming functions that we are snipping.
        if to_snip.contains(&id) {
            return;
        }

        let entry = func.entry_block();
        walrus::ir::dfs_pre_order_mut(&mut Replacer { to_snip }, func, entry);
    });
}

fn unexport_snipped_functions(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    let exports_to_snip: HashSet<walrus::ExportId> = module
        .exports
        .iter()
        .filter_map(|e| match e.item {
            walrus::ExportItem::Function(f) if to_snip.contains(&f) => Some(e.id()),
            _ => None,
        })
        .collect();

    for e in exports_to_snip {
        module.exports.delete(e);
    }
}

fn unimport_snipped_functions(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    let imports_to_snip: HashSet<walrus::ImportId> = module
        .imports
        .iter()
        .filter_map(|i| match i.kind {
            walrus::ImportKind::Function(f) if to_snip.contains(&f) => Some(i.id()),
            _ => None,
        })
        .collect();

    for i in imports_to_snip {
        module.imports.delete(i);
    }
}

fn snip_table_elements(module: &mut walrus::Module, to_snip: &HashSet<walrus::FunctionId>) {
    let mut unreachable_funcs: HashMap<walrus::TypeId, walrus::FunctionId> = Default::default();

    let make_unreachable_func = |ty: walrus::TypeId,
                                 types: &mut walrus::ModuleTypes,
                                 locals: &mut walrus::ModuleLocals,
                                 funcs: &mut walrus::ModuleFunctions|
     -> walrus::FunctionId {
        let ty = types.get(ty);
        let params = ty.params().to_vec();
        let locals: Vec<_> = params.iter().map(|ty| locals.add(*ty)).collect();
        let results = ty.results().to_vec();
        let mut builder = walrus::FunctionBuilder::new(types, &params, &results);
        builder.func_body().unreachable();
        builder.finish(locals, funcs)
    };

    for t in module.tables.iter_mut() {
        if let walrus::TableKind::Function(ref mut ft) = t.kind {
            let types = &mut module.types;
            let locals = &mut module.locals;
            let funcs = &mut module.funcs;

            ft.elements
                .iter_mut()
                .flat_map(|el| el)
                .filter(|f| to_snip.contains(f))
                .for_each(|el| {
                    let ty = funcs.get(*el).ty();
                    *el = *unreachable_funcs
                        .entry(ty)
                        .or_insert_with(|| make_unreachable_func(ty, types, locals, funcs));
                });

            ft.relative_elements
                .iter_mut()
                .flat_map(|(_, elems)| elems.iter_mut().filter(|f| to_snip.contains(f)))
                .for_each(|el| {
                    let ty = funcs.get(*el).ty();
                    *el = *unreachable_funcs
                        .entry(ty)
                        .or_insert_with(|| make_unreachable_func(ty, types, locals, funcs));
                });
        }
    }
}
