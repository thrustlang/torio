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

pub fn create(name: &str, project_type: crate::config::ProjectType) -> Result<(), String> {
    let valid_name: bool = !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        });

    if !valid_name || matches!(name, "." | "..") {
        return Err(format!(
            "Invalid project name '{name}'. Use ASCII letters, numbers, '-' or '_'."
        ));
    }

    let current_directory: std::path::PathBuf = std::env::current_dir()
        .map_err(|error| format!("Cannot determine the current directory: {error}."))?;
    let project_directory: std::path::PathBuf = current_directory.join(name);

    if project_directory.exists() {
        return Err(format!(
            "Path '{}' already exists.",
            project_directory.display()
        ));
    }

    let version: semver::Version = match crate::toolchain::active_version()? {
        Some(version) => version,
        None => crate::toolchain::install(None)?,
    };

    let sources_name: &str = match project_type {
        crate::config::ProjectType::Executable => "src",
        crate::config::ProjectType::Library => "lib",
    };
    let source_file_name: &str = match project_type {
        crate::config::ProjectType::Executable => "main.thrust",
        crate::config::ProjectType::Library => "lib.thrust",
    };
    let source: &str = match project_type {
        crate::config::ProjectType::Executable => {
            "import std::io;\n\nfn main() s32 @public {\n    io::print(\"Hello, World!\\n\");\n    return 0;\n}\n"
        }
        crate::config::ProjectType::Library => "fn doSomething() s32 @public {\n    return 0;\n}\n",
    };
    let sources_directory: std::path::PathBuf = project_directory.join(sources_name);
    let source_path: std::path::PathBuf = sources_directory.join(source_file_name);
    let manifest_path: std::path::PathBuf = project_directory.join("torio.yml");
    let manifest: String = crate::config::template(name, &version, project_type);

    std::fs::create_dir_all(&sources_directory).map_err(|error| {
        format!(
            "Cannot create project directory '{}': {error}.",
            sources_directory.display()
        )
    })?;

    if let Err(error) = std::fs::write(&source_path, source) {
        let _ = std::fs::remove_dir_all(&project_directory);

        return Err(format!(
            "Cannot write '{}': {error}.",
            source_path.display()
        ));
    }

    if let Err(error) = std::fs::write(&manifest_path, manifest) {
        let _ = std::fs::remove_dir_all(&project_directory);

        return Err(format!(
            "Cannot write '{}': {error}.",
            manifest_path.display()
        ));
    }

    println!(
        "Created Thrust project '{}' with toolchain v{version}.",
        project_directory.display()
    );
    Ok(())
}
