use clap::{App, SubCommand};

fn main() {
    let matches = App::new("Doctave CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .about(
            "An opinionated static site generator designed specifically \
               for technical documentation.",
        )
        .subcommand(SubCommand::with_name("init").about("Initialize a new project (start here!)"))
        .subcommand(
            SubCommand::with_name("build")
                .about("Builds your site from the project's Markdown files"),
        )
        .get_matches();

    let root_dir = std::env::current_dir().expect("Unable to determine current directory");

    let result = match matches.subcommand() {
        ("init", Some(_cmd)) => doctave::InitCommand::run(root_dir),
        ("build", Some(_cmd)) => doctave::BuildCommand::run(root_dir),
        _ => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}