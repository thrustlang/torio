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

pub fn execute(arguments: Vec<std::ffi::OsString>) -> Result<i32, String> {
    let version: semver::Version = crate::toolchain::active_version()?.ok_or_else(|| {
        "No active toolchain was found. Run 'torio toolchain install'.".to_string()
    })?;
    let compiler: std::path::PathBuf = crate::toolchain::compiler_path(&version)?;
    let status: std::process::ExitStatus = std::process::Command::new(&compiler)
        .args(arguments)
        .status()
        .map_err(|error| format!("Cannot execute '{}': {error}.", compiler.display()))?;

    Ok(status.code().unwrap_or(1))
}
