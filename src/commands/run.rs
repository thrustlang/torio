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

use crate::{commands, config};

pub fn execute(
    profile: config::BuildProfile,
    command_line_cc_args: Vec<String>,
    arguments: Vec<std::ffi::OsString>,
) -> Result<i32, String> {
    let output: commands::build::BuildOutput =
        commands::build::compile(profile, command_line_cc_args)?;

    if output.project_type != config::ProjectType::Executable {
        return Err("'torio run' is only available for executable projects.".into());
    }

    let status: std::process::ExitStatus = std::process::Command::new(&output.artifact)
        .args(arguments)
        .current_dir(&output.project_root)
        .status()
        .map_err(|error| format!("Cannot execute '{}': {error}.", output.artifact.display()))?;

    Ok(status.code().unwrap_or(1))
}
