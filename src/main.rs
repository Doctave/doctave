use std::path::PathBuf;

use bunt::termcolor::{ColorChoice, StandardStream};
use clap::{Parser, Subcommand};

/// An opinionated static site generator
/// designed specifically for technical documentation
#[derive(Parser)]
#[clap(name = "Doctave", version)]
struct Cli {
    /// Disable terminal color output
    #[clap(long, global = true)]
    no_color: bool,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Initialize a new project (start here!)
    Init {
        /// An optional custom root directory for your documentation (defaults to docs/)
        #[clap(long)]
        docs_dir: Option<PathBuf>,
    },

    /// Build your site from the project's Markdown files
    Build {
        /// Build the site in release mode
        #[clap(long)]
        release: bool,

        /// Don't return an error if there are failed checks
        #[clap(long)]
        allow_failed_checks: bool,
    },

    /// Start a live reloading development server
    /// to serve your documentation site
    Serve {
        /// Port used to serve the documentation site.
        /// Must be a positive integer.
        #[clap(short, long)]
        port: Option<u32>,
    },
}

fn main() {
    let cli = Cli::parse();
    let no_color = cli.no_color;
    let result = match cli.command {
        Command::Init { docs_dir } => init(no_color, docs_dir),
        Command::Build {
            release,
            allow_failed_checks,
        } => build(no_color, release, allow_failed_checks),
        Command::Serve { port } => serve(no_color, port),
    };

    let color_choice = if no_color {
        ColorChoice::Never
    } else {
        ColorChoice::Auto
    };
    let mut out = StandardStream::stdout(color_choice);

    if let Err(e) = result {
        bunt::writeln!(out, "{$red}ERROR:{/$} {}", e).unwrap();
        std::process::exit(1);
    }
}

fn init(no_color: bool, docs_dir: Option<PathBuf>) -> doctave::Result<()> {
    let root_dir = std::env::current_dir().expect("Unable to determine current directory");
    doctave::InitCommand::run(root_dir, !no_color, docs_dir)
}

fn build(no_color: bool, release: bool, allow_failed_checks: bool) -> doctave::Result<()> {
    let project_dir = doctave::config::project_root().unwrap_or_else(|| {
        println!("Could not find a doctave project in this directory, or its parents.");
        std::process::exit(1);
    });

    let mut config = doctave::Config::load(&project_dir)?;
    if release {
        config.set_build_mode(doctave::BuildMode::Release);
    }

    if no_color {
        config.disable_colors();
    }

    if allow_failed_checks {
        config.set_allow_failed_checks();
    }

    doctave::BuildCommand::run(config)
}

fn serve(no_color: bool, port: Option<u32>) -> doctave::Result<()> {
    let project_dir = doctave::config::project_root().unwrap_or_else(|| {
        println!("Could not find a doctave project in this directory, or its parents.");
        std::process::exit(1);
    });

    let options = doctave::ServeOptions { port };
    let mut config = doctave::Config::load(&project_dir)?;

    if no_color {
        config.disable_colors();
    }

    doctave::ServeCommand::run(options, config)
}
