use clap::{App, Arg, ArgMatches, SubCommand};

fn main() {
    let matches = App::new("Doctave CLI")
        .version(env!("CARGO_PKG_VERSION"))
        .about(
            "An opinionated static site generator designed specifically \
               for technical documentation.",
        )
        .arg(
            Arg::with_name("no-color")
                .long("no-color")
                .help("Disable terminal color output")
                .global(true),
        )
        .subcommand(SubCommand::with_name("init").about("Initialize a new project (start here!)"))
        .subcommand(
            SubCommand::with_name("build")
                .about("Builds your site from the project's Markdown files")
                .arg(
                    Arg::with_name("release")
                        .long("release")
                        .help("Build the site in release mode"),
                ),
        )
        .subcommand(
            SubCommand::with_name("serve")
                .about(
                    "Starts a live reloading development server to serve your documentation site",
                )
                .arg(
                    Arg::with_name("port")
                        .long("port")
                        .short("p")
                        .takes_value(true)
                        .value_name("PORT")
                        .help(
                            "Port used to serve the documentation site. \
                             Must be a positive integer.",
                        )
                        .validator(|p| match p.parse::<u32>() {
                            Ok(_) => Ok(()),
                            Err(e) => Err(e.to_string()),
                        }),
                ),
        )
        .get_matches();

    let result = match matches.subcommand() {
        ("init", Some(cmd)) => init(cmd),
        ("build", Some(cmd)) => build(cmd),
        ("serve", Some(cmd)) => serve(cmd),
        _ => Ok(()),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn init(cmd: &ArgMatches) -> doctave::Result<()> {
    let root_dir = std::env::current_dir().expect("Unable to determine current directory");
    doctave::InitCommand::run(root_dir, !cmd.is_present("no-color"))
}

fn build(cmd: &ArgMatches) -> doctave::Result<()> {
    let project_dir = doctave::config::project_root().unwrap_or_else(|| {
        println!("Could not find a doctave project in this directory, or its parents.");
        std::process::exit(1);
    });

    let mut config = doctave::Config::load(&project_dir)?;
    if cmd.is_present("release") {
        config.set_build_mode(doctave::BuildMode::Release);
    }

    if cmd.is_present("no-color") {
        config.disable_colors();
    }

    doctave::BuildCommand::run(config)
}

fn serve(cmd: &ArgMatches) -> doctave::Result<()> {
    let project_dir = doctave::config::project_root().unwrap_or_else(|| {
        println!("Could not find a doctave project in this directory, or its parents.");
        std::process::exit(1);
    });

    let mut options = doctave::ServeOptions::default();
    let mut config = doctave::Config::load(&project_dir)?;

    if let Some(p) = cmd.value_of("port") {
        options.port = Some(p.parse::<u32>().unwrap());
    }

    if cmd.is_present("no-color") {
        config.disable_colors();
    }

    doctave::ServeCommand::run(options, config)
}
