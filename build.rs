use clap::IntoApp;
use clap_generate::{generate_to, generators::*};

#[allow(dead_code)]
mod sixel {
    include!("src/sixel/lib.rs");
}

#[allow(dead_code)]
mod cli {
    include!("src/cli.rs");
}
mod error {
    use crate::sixel;
    include!("src/error.rs");
}
mod logger {
    include!("src/logger.rs");
}

#[allow(dead_code)]
mod seek {
    include!("src/seek.rs");
}
mod subcommands {
    include!("src/subcommands.rs");
}

fn main() {
    let mut app = cli::Opts::into_app();
    let outdir = env!("CARGO_MANIFEST_DIR");
    generate_to::<Bash, _, _>(&mut app, "davis", outdir);
}
