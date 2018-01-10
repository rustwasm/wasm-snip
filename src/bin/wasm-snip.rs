extern crate clap;
extern crate failure;
extern crate parity_wasm;
extern crate wasm_snip;

use parity_wasm::elements::{self, Serialize};
use std::io;
use std::path;
use std::process;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("error: {}", e);
        for c in e.causes().skip(1) {
            eprintln!("  caused by {}", c);
        }
        eprintln!("{}", e.backtrace());
        process::exit(1)
    }
}

fn try_main() -> Result<(), failure::Error> {
    let matches = parse_args();

    let mut opts = wasm_snip::Options::default();
    opts.input = path::PathBuf::from(matches.value_of("input").unwrap());
    opts.functions = matches
        .values_of("function")
        .unwrap()
        .map(|f| f.to_string())
        .collect();

    let module = wasm_snip::snip(opts)?;

    if let Some(output) = matches.value_of("output") {
        elements::serialize_to_file(output, module)?;
    } else {
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        module.serialize(&mut stdout)?;
    }

    Ok(())
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
        .arg(
            clap::Arg::with_name("function")
                .required(true)
                .multiple(true)
                .help("The function(s) to snip."),
        )
        .get_matches()
}
