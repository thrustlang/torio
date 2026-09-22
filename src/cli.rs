/*

    Copyright (C) 2026  Stevens Benavides

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "torio",
    version,
    about = "Thrust Programming Language toolchain and project manager",
    long_about = "Torio creates Thrust projects, manages versioned compiler toolchains, builds executables and libraries, and runs compiled programs.",
    after_help = "Examples:\n  torio new hello\n  torio build --release\n  torio run -- argument\n  torio thrustc --version\n  torio toolchain install"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Create a new executable or library project.
    New(NewArguments),

    /// Compile the current project into dist/dev or dist/release.
    Build(BuildArguments),

    /// Build and run the current executable project.
    Run(RunArguments),

    /// Invoke the compiler from the active toolchain directly.
    Thrustc(ThrustcArguments),

    /// Install, update, select, list or remove compiler toolchains.
    Toolchain(ToolchainArguments),

    /// Print the installed Torio version.
    Version,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  torio new hello\n  torio new math --lib\n  torio new application --executable"
)]
struct NewArguments {
    /// Project name and destination directory.
    #[arg(value_name = "NAME")]
    name: String,

    /// Create an executable project. This is the default project type.
    #[arg(long, conflicts_with = "library")]
    executable: bool,

    /// Create a dynamic library project under lib/.
    #[arg(long = "lib")]
    library: bool,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  torio build\n  torio build --release\n  torio build --cc-arg -lm --cc-arg -lpthread"
)]
struct BuildArguments {
    /// Use the release profile and write the artifact under dist/release/.
    #[arg(long)]
    release: bool,

    /// Forward one argument to the C compiler through thrustc -cc-args.
    #[arg(long = "cc-arg", action = clap::ArgAction::Append, allow_hyphen_values = true)]
    cc_args: Vec<String>,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  torio run\n  torio run --release\n  torio run -- input.txt --verbose\n  torio run --cc-arg -lm -- value"
)]
struct RunArguments {
    /// Use the release profile before running the program.
    #[arg(long)]
    release: bool,

    /// Forward one argument to the C compiler through thrustc -cc-args.
    #[arg(long = "cc-arg", action = clap::ArgAction::Append, allow_hyphen_values = true)]
    cc_args: Vec<String>,

    /// Arguments passed unchanged to the compiled program after --.
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<std::ffi::OsString>,
}

#[derive(Debug, Args)]
#[command(
    disable_help_flag = true,
    disable_version_flag = true,
    trailing_var_arg = true,
    after_help = "Examples:\n  torio thrustc --help\n  torio thrustc --version\n  torio thrustc -emit llvm-ir main.thrust"
)]
struct ThrustcArguments {
    /// Arguments passed unchanged to thrustc from the active toolchain.
    #[arg(allow_hyphen_values = true)]
    arguments: Vec<std::ffi::OsString>,
}

#[derive(Debug, Args)]
#[command(
    after_help = "Examples:\n  torio toolchain install\n  torio toolchain install 0.2.2\n  torio toolchain update\n  torio toolchain list\n  torio toolchain use 0.2.2"
)]
struct ToolchainArguments {
    #[command(subcommand)]
    command: ToolchainCommand,
}

#[derive(Debug, Subcommand)]
enum ToolchainCommand {
    /// Install and activate a complete compiler, LSP and VS Code toolchain.
    #[command(after_help = "Examples:\n  torio toolchain install\n  torio toolchain install 0.2.2")]
    Install {
        /// Exact version to install. The latest stable version is used when omitted.
        #[arg(value_name = "VERSION")]
        version: Option<String>,
    },

    /// Update the active toolchain and Torio to the latest stable releases.
    #[command(after_help = "Example:\n  torio toolchain update")]
    Update,

    /// List installed toolchain versions and mark the active version.
    #[command(after_help = "Example:\n  torio toolchain list")]
    List,

    /// Activate an already installed toolchain version.
    #[command(after_help = "Example:\n  torio toolchain use 0.2.2")]
    Use {
        /// Installed version to activate.
        #[arg(value_name = "VERSION")]
        version: String,
    },

    /// Remove an installed toolchain version that is not active.
    #[command(after_help = "Example:\n  torio toolchain remove 0.2.1")]
    Remove {
        /// Installed version to remove.
        #[arg(value_name = "VERSION")]
        version: String,
    },
}

pub fn execute() -> Result<i32, String> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Command::New(arguments) => {
            let project_type: crate::config::ProjectType = if arguments.library {
                crate::config::ProjectType::Library
            } else {
                crate::config::ProjectType::Executable
            };

            crate::commands::project::create(&arguments.name, project_type)?;
            Ok(0)
        }
        Command::Build(arguments) => {
            let profile: crate::config::BuildProfile = if arguments.release {
                crate::config::BuildProfile::Release
            } else {
                crate::config::BuildProfile::Dev
            };

            crate::commands::build::execute(profile, arguments.cc_args)?;
            Ok(0)
        }
        Command::Run(arguments) => {
            let profile: crate::config::BuildProfile = if arguments.release {
                crate::config::BuildProfile::Release
            } else {
                crate::config::BuildProfile::Dev
            };

            crate::commands::run::execute(profile, arguments.cc_args, arguments.arguments)
        }
        Command::Thrustc(arguments) => crate::commands::compiler::execute(arguments.arguments),
        Command::Toolchain(arguments) => match arguments.command {
            ToolchainCommand::Install { version } => {
                crate::toolchain::install(version.as_deref())?;
                Ok(0)
            }
            ToolchainCommand::Update => {
                crate::toolchain::update()?;
                Ok(0)
            }
            ToolchainCommand::List => {
                crate::toolchain::list()?;
                Ok(0)
            }
            ToolchainCommand::Use { version } => {
                crate::toolchain::use_version(&version)?;
                Ok(0)
            }
            ToolchainCommand::Remove { version } => {
                crate::toolchain::remove(&version)?;
                Ok(0)
            }
        },
        Command::Version => {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
    }
}
