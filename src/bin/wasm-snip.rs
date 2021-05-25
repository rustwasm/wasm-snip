use clap::ArgMatches;
use failure::ResultExt;
use std::fs;
use std::io::{self, Write};
use std::process;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("error: {}", e);
        for c in e.iter_chain().skip(1) {
            eprintln!("  caused by {}", c);
        }
        eprintln!("{}", e.backtrace());
        process::exit(1)
    }
}

fn get_values(matches: &ArgMatches, name: &str) -> Vec<String> {
    matches
        .values_of(name)
        .map(|fs| fs.map(|f| f.to_string()).collect())
        .unwrap_or(vec![])
}

fn try_main() -> Result<(), failure::Error> {
    let matches = parse_args();

    let mut opts = wasm_snip::Options::default();

    opts.functions = get_values(&matches, "function");
    opts.patterns = get_values(&matches, "pattern");
    opts.kept_exports = get_values(&matches, "kept_export");
    opts.kept_export_patterns = get_values(&matches, "kept_export_pattern");

    opts.snip_rust_fmt_code = matches.is_present("snip_rust_fmt_code");
    opts.snip_rust_panicking_code = matches.is_present("snip_rust_panicking_code");
    opts.skip_producers_section = matches.is_present("skip_producers_section");

    let config = walrus_config_from_options(&opts);
    let path = matches.value_of("input").unwrap();
    let buf = fs::read(&path).with_context(|_| format!("failed to read file {}", path))?;
    let mut module = config.parse(&buf)?;

    wasm_snip::snip(&mut module, opts).context("failed to snip functions from wasm module")?;

    if let Some(output) = matches.value_of("output") {
        module
            .emit_wasm_file(output)
            .with_context(|_| format!("failed to emit snipped wasm to {}", output))?;
    } else {
        let wasm = module.emit_wasm();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        stdout
            .write_all(&wasm)
            .context("failed to write wasm to stdout")?;
    }

    Ok(())
}

fn walrus_config_from_options(options: &wasm_snip::Options) -> walrus::ModuleConfig {
    let mut config = walrus::ModuleConfig::new();
    config.generate_producers_section(!options.skip_producers_section);
    config
}

fn parse_args() -> clap::ArgMatches<'static> {
    clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .long_about(
            "
`wasm-snip` replaces a WebAssembly function's body with an `unreachable`.

Maybe you know that some function will never be called at runtime, but the
compiler can't prove that at compile time? Snip it! Then run `wasm-gc`[0] again
and all the functions it transitively called (which could also never be called
at runtime) will get removed too.

Very helpful when shrinking the size of WebAssembly binaries!

[0]: https://github.com/alexcrichton/wasm-gc
",
        )
        .arg(
            clap::Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .help("The path to write the output wasm file to. Defaults to stdout."),
        )
        .arg(
            clap::Arg::with_name("input")
                .required(true)
                .help("The input wasm file containing the function(s) to snip."),
        )
        .arg(clap::Arg::with_name("function").multiple(true).help(
            "The specific function(s) to snip. These must match \
             exactly. Use the -p flag for fuzzy matching.",
        ))
        .arg(
            clap::Arg::with_name("pattern")
                .required(false)
                .multiple(true)
                .short("p")
                .long("pattern")
                .takes_value(true)
                .help("Snip any function that matches the given regular expression."),
        )
        .arg(
            clap::Arg::with_name("kept_export")
                .required(false)
                .multiple(true)
                .short("k")
                .long("kept-export")
                .takes_value(true)
                .help("Snip exports not included. Matches exactly."),
        )
        .arg(
            clap::Arg::with_name("kept_export_pattern")
                .required(false)
                .multiple(true)
                .short("x")
                .long("kept-export-pattern")
                .takes_value(true)
                .help("Snip exports not included. Matches exports with regular expression."),
        )
        .arg(
            clap::Arg::with_name("snip_rust_fmt_code")
                .required(false)
                .long("snip-rust-fmt-code")
                .help("Snip Rust's `std::fmt` and `core::fmt` code."),
        )
        .arg(
            clap::Arg::with_name("snip_rust_panicking_code")
                .required(false)
                .long("snip-rust-panicking-code")
                .help("Snip Rust's `std::panicking` and `core::panicking` code."),
        )
        .arg(
            clap::Arg::with_name("skip_producers_section")
                .required(false)
                .long("skip-producers-section")
                .help("Do not emit the 'producers' custom section."),
        )
        .get_matches()
}
