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
        .subcommand(
            SubCommand::with_name("serve").about(
                "Starts a live reloading development server to serve your documentation site",
                ),
                )
        .get_matches();

    let result = match matches.subcommand() {
        ("init", Some(_cmd)) => init(),
        ("build", Some(_cmd)) => build(),
        ("serve", Some(_cmd)) => serve(),
        _ => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn init() -> std::io::Result<()> {
    let root_dir = std::env::current_dir().expect("Unable to determine current directory");
    doctave::InitCommand::run(root_dir)
}

fn build() -> std::io::Result<()> {
    let project_dir = doctave::config::project_root().unwrap_or_else(|| {
        println!("Could not find a doctave project in this directory, or its parents.");
        std::process::exit(1);
    });

    let config = doctave::config::Config::load(&project_dir)?;

    doctave::BuildCommand::run(config)
}

fn serve() -> std::io::Result<()> {
    let project_dir = doctave::config::project_root().unwrap_or_else(|| {
        println!("Could not find a doctave project in this directory, or its parents.");
        std::process::exit(1);
    });

    let config = doctave::config::Config::load(&project_dir)?;

    doctave::ServeCommand::run(config)
}
