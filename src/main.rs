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

pub mod cli;
pub mod commands;
pub mod config;
pub mod toolchain;

fn main() {
    let executable: std::path::PathBuf = std::env::current_exe().unwrap_or_default();
    let executable_name: String = executable
        .file_stem()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());

    let shim_names: [&str; 4] = [
        "thrustc",
        "thrustc-stripped",
        "thrustc_lsp",
        "thrustc_lsp-stripped",
    ];

    if shim_names.contains(&executable_name.as_str()) {
        let exit_code: i32 = toolchain::dispatch_shim(&executable_name).unwrap_or_else(|error| {
            eprintln!("error: {error}");
            1
        });

        std::process::exit(exit_code);
    }

    let exit_code: i32 = cli::execute().unwrap_or_else(|error| {
        eprintln!("error: {error}");
        1
    });

    std::process::exit(exit_code);
}
