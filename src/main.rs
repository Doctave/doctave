use clap::{App, Arg, SubCommand};

fn main() {
    let matches = App::new("Doctave CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .about(
            "An opinionated static site generator designed specifically \
               for technical documentation.",
        )
        .subcommand(
            SubCommand::with_name("init")
                .about("Initialize a new project (start here!)")
        )
        .get_matches();
}
