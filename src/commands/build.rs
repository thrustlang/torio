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

#![allow(clippy::vec_init_then_push)]

use std::io::{BufRead, IsTerminal};

#[derive(Debug)]
pub struct BuildOutput {
    pub artifact: std::path::PathBuf,
    pub project_root: std::path::PathBuf,
    pub project_type: crate::config::ProjectType,
}

pub fn compile(
    profile: crate::config::BuildProfile,
    command_line_cc_args: Vec<String>,
) -> Result<BuildOutput, String> {
    let (project_root, manifest): (std::path::PathBuf, crate::config::Manifest) =
        crate::config::load()?;
    let (configured_profile, profile_name): (&crate::config::ProfileConfiguration, &str) =
        match profile {
            crate::config::BuildProfile::Dev => (&manifest.profiles.dev, "dev"),
            crate::config::BuildProfile::Release => (&manifest.profiles.release, "release"),
        };
    let compiler: std::path::PathBuf =
        crate::toolchain::compiler_path(&manifest.toolchain.version)?;
    let sources_root: std::path::PathBuf = project_root.join(&manifest.sources.directory);
    let mut pending_directories: Vec<std::path::PathBuf> = vec![sources_root.clone()];
    let mut sources: Vec<std::path::PathBuf> = Vec::new();

    while let Some(directory) = pending_directories.pop() {
        let entries: std::fs::ReadDir = std::fs::read_dir(&directory).map_err(|error| {
            format!(
                "Cannot read source directory '{}': {error}.",
                directory.display()
            )
        })?;

        for entry in entries {
            let entry: std::fs::DirEntry = entry.map_err(|error| {
                format!(
                    "Cannot read an entry from '{}': {error}.",
                    directory.display()
                )
            })?;
            let path: std::path::PathBuf = entry.path();
            let file_type: std::fs::FileType = entry
                .file_type()
                .map_err(|error| format!("Cannot inspect '{}': {error}.", path.display()))?;

            if file_type.is_symlink() {
                return Err(format!(
                    "Symbolic links are not allowed inside the source tree: '{}'.",
                    path.display()
                ));
            }

            if file_type.is_dir() {
                pending_directories.push(path);
                continue;
            }

            let extension: Option<&str> = path.extension().and_then(std::ffi::OsStr::to_str);

            if matches!(extension, Some("thrust" | "tht" | "🐦")) {
                sources.push(path);
            }
        }
    }

    sources.sort();

    if sources.is_empty() {
        return Err(format!(
            "No Thrust source files were found in '{}'.",
            sources_root.display()
        ));
    }

    let dist_directory: std::path::PathBuf = project_root.join("dist");

    if dist_directory.exists() {
        let dist_metadata: std::fs::Metadata = std::fs::symlink_metadata(&dist_directory)
            .map_err(|error| format!("Cannot inspect '{}': {error}.", dist_directory.display()))?;

        if dist_metadata.file_type().is_symlink() {
            return Err(format!(
                "Build output directory '{}' cannot be a symbolic link.",
                dist_directory.display()
            ));
        }
    }

    let output_directory: std::path::PathBuf = dist_directory.join(profile_name);
    let build_directory: std::path::PathBuf = output_directory.join("build");

    std::fs::create_dir_all(&build_directory)
        .map_err(|error| format!("Cannot create '{}': {error}.", build_directory.display()))?;

    let artifact_name: String = match manifest.project.project_type {
        crate::config::ProjectType::Executable if cfg!(windows) => {
            format!("{}.exe", manifest.project.name)
        }
        crate::config::ProjectType::Executable => manifest.project.name.clone(),
        crate::config::ProjectType::Library if cfg!(target_os = "windows") => {
            format!("{}.dll", manifest.project.name)
        }
        crate::config::ProjectType::Library if cfg!(target_os = "macos") => {
            format!("lib{}.dylib", manifest.project.name)
        }
        crate::config::ProjectType::Library => format!("lib{}.so", manifest.project.name),
    };
    let artifact: std::path::PathBuf = output_directory.join(artifact_name);

    if artifact.exists() {
        std::fs::remove_file(&artifact)
            .map_err(|error| format!("Cannot replace '{}': {error}.", artifact.display()))?;
    }

    let mut cc_args: Vec<String> = Vec::new();

    cc_args.extend(manifest.linker.cc_args.iter().cloned());
    cc_args.extend(configured_profile.cc_args.iter().cloned());
    cc_args.extend(command_line_cc_args);

    for argument in cc_args.iter() {
        let reserves_output: bool = argument == "-o"
            || argument == "--output"
            || argument.starts_with("-o=")
            || argument.starts_with("--output=")
            || (argument.starts_with("-o") && argument.len() > 2);

        if reserves_output {
            return Err(format!(
                "The C compiler argument '{argument}' is reserved. Torio controls output paths under dist/."
            ));
        }
    }

    if manifest.project.project_type == crate::config::ProjectType::Library {
        if cfg!(target_os = "macos") {
            cc_args.push("-dynamiclib".into());
        } else {
            cc_args.push("-shared".into());
        }
    }

    cc_args.push("-o".into());
    cc_args.push(artifact.to_string_lossy().to_string());

    let mut serialized_cc_args: String = String::new();

    for argument in cc_args.iter() {
        if !serialized_cc_args.is_empty() {
            serialized_cc_args.push(';');
        }

        serialized_cc_args.push('"');

        for character in argument.chars() {
            if character == '\\' || character == '"' {
                serialized_cc_args.push('\\');
            }

            serialized_cc_args.push(character);
        }

        serialized_cc_args.push('"');
    }

    let mut compiler_arguments: Vec<String> = Vec::new();

    compiler_arguments.push("-build-dir".into());
    compiler_arguments.push(build_directory.to_string_lossy().to_string());
    compiler_arguments.push("-opt".into());
    compiler_arguments.push(configured_profile.optimization.clone());

    if configured_profile.debug {
        compiler_arguments.push("-dbg".into());
    }

    if !manifest.compiler.warnings {
        compiler_arguments.push("--disable-all-warnings".into());
    }

    if manifest.project.project_type == crate::config::ProjectType::Library && cfg!(unix) {
        compiler_arguments.push("-reloc-model".into());
        compiler_arguments.push("pic".into());
    }

    let base_compiler_arguments: Vec<String> =
        crate::config::compiler_arguments(&manifest.compiler);

    compiler_arguments.extend(base_compiler_arguments);
    compiler_arguments.extend(configured_profile.compiler_args.iter().cloned());

    match manifest.linker.compiler.as_str() {
        "clang" => {
            compiler_arguments.push("-link-with-clang".into());

            let path: String = manifest
                .linker
                .path
                .as_ref()
                .map_or_else(|| "clang".into(), |path| path.to_string_lossy().to_string());

            compiler_arguments.push(path);
        }
        "gcc" => {
            compiler_arguments.push("-link-with-gcc".into());

            let path: String = manifest
                .linker
                .path
                .as_ref()
                .map_or_else(|| "gcc".into(), |path| path.to_string_lossy().to_string());

            compiler_arguments.push(path);
        }
        _ => {}
    }

    for source in sources.iter() {
        compiler_arguments.push(source.to_string_lossy().to_string());
    }

    compiler_arguments.push("-cc-args".into());
    compiler_arguments.push(serialized_cc_args);

    let mut command: std::process::Command = std::process::Command::new(&compiler);

    command.args(&compiler_arguments);
    command.current_dir(&project_root);
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());

    let mut child: std::process::Child = command
        .spawn()
        .map_err(|error| format!("Cannot execute '{}': {error}.", compiler.display()))?;
    let stdout: std::process::ChildStdout = child
        .stdout
        .take()
        .ok_or_else(|| "Cannot capture thrustc stdout.".to_string())?;
    let stderr: std::process::ChildStderr = child
        .stderr
        .take()
        .ok_or_else(|| "Cannot capture thrustc stderr.".to_string())?;
    let stderr_reader: std::thread::JoinHandle<Result<String, std::io::Error>> =
        std::thread::spawn(move || {
            let mut reader: std::io::BufReader<std::process::ChildStderr> =
                std::io::BufReader::new(stderr);
            let mut content: String = String::new();

            std::io::Read::read_to_string(&mut reader, &mut content)?;
            Ok(content)
        });
    let interactive: bool = std::io::stdout().is_terminal();
    let mut spinner: Option<terminal_spinners::SpinnerHandle> = if interactive {
        let first_source: String = sources[0].display().to_string();

        Some(
            terminal_spinners::SpinnerBuilder::new()
                .spinner(&terminal_spinners::DOTS)
                .text(format!(" Compiling {first_source}"))
                .start(),
        )
    } else {
        None
    };
    let stdout_reader: std::io::BufReader<std::process::ChildStdout> =
        std::io::BufReader::new(stdout);

    for line in stdout_reader.lines() {
        let line: String = line.map_err(|error| format!("Cannot read thrustc output: {error}."))?;

        if let Some(handle) = spinner.as_ref() {
            if line.contains("Compilation")
                && (line.contains("STARTED") || line.contains("FINISHED"))
            {
                let file: &str = line.split_whitespace().last().unwrap_or("source");

                handle.text(format!(" Compiling {file}"));
            } else if line.contains("Linking") {
                handle.text(" Linking project");
            }
        }
    }

    let status: std::process::ExitStatus = child
        .wait()
        .map_err(|error| format!("Cannot wait for thrustc: {error}."))?;
    let stderr: String = stderr_reader
        .join()
        .map_err(|_| "The thrustc stderr reader thread failed.".to_string())?
        .map_err(|error| format!("Cannot read thrustc diagnostics: {error}."))?;

    if !status.success() || !artifact.is_file() {
        if let Some(handle) = spinner.take() {
            handle.error();
        }

        if !stderr.trim().is_empty() {
            eprintln!("{}", stderr.trim_end());
        }

        if status.success() {
            return Err(format!(
                "thrustc did not produce expected artifact '{}'.",
                artifact.display()
            ));
        }

        return Err(format!("thrustc failed with status {status}."));
    }

    if let Some(handle) = spinner.take() {
        handle.done();
    }

    if !stderr.trim().is_empty() {
        eprintln!("{}", stderr.trim_end());
    }

    Ok(BuildOutput {
        artifact,
        project_root,
        project_type: manifest.project.project_type,
    })
}

pub fn execute(
    profile: crate::config::BuildProfile,
    command_line_cc_args: Vec<String>,
) -> Result<(), String> {
    let output: BuildOutput = self::compile(profile, command_line_cc_args)?;

    println!("Built {}", output.artifact.display());
    Ok(())
}
