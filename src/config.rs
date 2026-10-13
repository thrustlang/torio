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

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    Executable,
    Library,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BuildProfile {
    Dev,
    Release,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema: u32,
    pub project: ProjectConfiguration,
    pub toolchain: ToolchainConfiguration,
    pub sources: SourcesConfiguration,
    pub compiler: CompilerConfiguration,
    pub linker: LinkerConfiguration,
    pub profiles: ProfilesConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfiguration {
    pub name: String,
    pub version: semver::Version,
    #[serde(rename = "type")]
    pub project_type: ProjectType,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolchainConfiguration {
    pub version: semver::Version,
    #[serde(default, rename = "compiler-variant")]
    compiler_variant: Option<crate::variant::CompilerVariant>,
}

impl ToolchainConfiguration {
    pub fn new(
        version: semver::Version,
        compiler_variant: Option<crate::variant::CompilerVariant>,
    ) -> Self {
        Self {
            version,
            compiler_variant,
        }
    }
}

impl ToolchainConfiguration {
    pub fn get_compiler_variant(&self) -> Option<crate::variant::CompilerVariant> {
        self.compiler_variant
    }
}

impl ToolchainConfiguration {
    pub fn get_compiler_variant_mut(
        &mut self,
    ) -> &mut Option<crate::variant::CompilerVariant> {
        &mut self.compiler_variant
    }
}

impl ToolchainConfiguration {
    pub fn set_compiler_variant(
        &mut self,
        compiler_variant: Option<crate::variant::CompilerVariant>,
    ) {
        self.compiler_variant = compiler_variant;
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcesConfiguration {
    pub directory: std::path::PathBuf,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct CompilerConfiguration {
    #[serde(default)]
    pub warnings: bool,
    #[serde(default)]
    pub general: Vec<String>,
    #[serde(default)]
    pub target: Vec<String>,
    #[serde(default)]
    pub optimization: Vec<String>,
    #[serde(default)]
    pub code_generation: Vec<String>,
    #[serde(default)]
    pub debug: Vec<String>,
    #[serde(default)]
    pub sanitizers: Vec<String>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    #[serde(default)]
    pub emission: Vec<String>,
    #[serde(default)]
    pub experimental: Vec<String>,
    #[serde(default)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct LinkerConfiguration {
    #[serde(default)]
    pub compiler: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub cc_args: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilesConfiguration {
    pub dev: ProfileConfiguration,
    pub release: ProfileConfiguration,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub struct ProfileConfiguration {
    pub optimization: String,
    pub debug: bool,
    #[serde(default)]
    pub compiler_args: Vec<String>,
    #[serde(default)]
    pub cc_args: Vec<String>,
}

pub fn template(name: &str, version: &semver::Version, project_type: ProjectType) -> String {
    let project_type_name: &str = match project_type {
        ProjectType::Executable => "executable",
        ProjectType::Library => "library",
    };

    let sources_directory: &str = match project_type {
        ProjectType::Executable => "src",
        ProjectType::Library => "lib",
    };

    format!(
        r#"# Torio project manifest format.
schema: 1

project:
  name: {name}
  version: 0.1.0
  type: {project_type_name} # executable | library

toolchain:
  version: {version}
  # compiler-variant: release # release | stripped

sources:
  directory: {sources_directory}

compiler:
  warnings: false
  general: [] # Example: ["-mode", "stable", "-abi", "system-v"]
  target: [] # Example: ["-target", "x86_64", "-cpu", "haswell"]
  optimization: [] # Example: ["--opt-passes", "-p{{instcombine,simplifycfg}}"]
  code-generation: [] # Example: ["-reloc-model", "pic", "-code-model", "small"]
  debug: [] # Example: ["-dbg", "-dbg-dwarf-version", "v5"]
  sanitizers: [] # Example: ["--sanitizer", "address"]
  diagnostics: [] # Example: ["--enable-ansi-color", "--export-compiler-errors"]
  emission: [] # Example: ["-emit", "llvm-ir"]
  experimental: [] # Example: ["-mode", "unstable"]
  extra-args: [] # Example: ["--disable-frame-pointer"]

linker:
  compiler: auto # auto | clang | gcc
  path: null # Example: "/usr/bin/clang"
  cc-args: [] # Example: ["-lm", "-lpthread", "-L/usr/local/lib"]

# Project configuration above is shared. Profiles only override build behavior.
profiles:
  dev:
    optimization: O0 # Example: O1, O2, O3, Oz
    debug: true
    compiler-args: [] # Example: ["-cpu", "haswell"]
    cc-args: [] # Example: ["-lm"]
  release:
    optimization: O3
    debug: false
    compiler-args: [] # Example: ["--disable-frame-pointer"]
    cc-args: [] # Example: ["-flto"]
"#
    )
}

pub fn load() -> Result<(std::path::PathBuf, Manifest), String> {
    let current_directory: std::path::PathBuf = std::env::current_dir()
        .map_err(|error| format!("Cannot determine the current directory: {error}."))?;

    let mut candidate: std::path::PathBuf = current_directory;

    loop {
        let manifest_path: std::path::PathBuf = candidate.join("torio.yml");

        if manifest_path.is_file() {
            let content: String = std::fs::read_to_string(&manifest_path)
                .map_err(|error| format!("Cannot read '{}': {error}.", manifest_path.display()))?;

            let manifest: Manifest = serde_yaml::from_str(&content).map_err(|error| {
                format!(
                    "Invalid Torio manifest '{}': {error}.",
                    manifest_path.display()
                )
            })?;

            self::validate(&candidate, &manifest)?;
            return Ok((candidate, manifest));
        }

        if !candidate.pop() {
            break;
        }
    }

    Err("No 'torio.yml' was found in the current directory or its parents.".into())
}

pub fn compiler_arguments(configuration: &CompilerConfiguration) -> Vec<String> {
    let mut arguments: Vec<String> = Vec::new();

    arguments.extend(configuration.general.iter().cloned());
    arguments.extend(configuration.target.iter().cloned());
    arguments.extend(configuration.optimization.iter().cloned());
    arguments.extend(configuration.code_generation.iter().cloned());
    arguments.extend(configuration.debug.iter().cloned());
    arguments.extend(configuration.sanitizers.iter().cloned());
    arguments.extend(configuration.diagnostics.iter().cloned());
    arguments.extend(configuration.emission.iter().cloned());
    arguments.extend(configuration.experimental.iter().cloned());
    arguments.extend(configuration.extra_args.iter().cloned());
    arguments
}

fn validate(root: &std::path::Path, manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != 1 {
        return Err(format!(
            "Unsupported torio.yml schema '{}'. This Torio version supports schema 1.",
            manifest.schema
        ));
    }

    let name: &str = manifest.project.name.as_str();
    let valid_name: bool = !name.is_empty()
        && name.chars().all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '-'
        });

    if !valid_name {
        return Err(format!("Invalid project name '{name}'."));
    }

    if manifest.sources.directory.is_absolute()
        || manifest
            .sources
            .directory
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(format!(
            "Source directory '{}' must be a relative path contained by the project.",
            manifest.sources.directory.display()
        ));
    }

    let root: std::path::PathBuf = root
        .canonicalize()
        .map_err(|error| format!("Cannot resolve project root '{}': {error}.", root.display()))?;
    let sources: std::path::PathBuf = root.join(&manifest.sources.directory);

    if !sources.is_dir() {
        return Err(format!(
            "The source directory '{}' does not exist.",
            sources.display()
        ));
    }

    let source_metadata: std::fs::Metadata = std::fs::symlink_metadata(&sources)
        .map_err(|error| format!("Cannot inspect '{}': {error}.", sources.display()))?;

    if source_metadata.file_type().is_symlink() {
        return Err(format!(
            "Source directory '{}' cannot be a symbolic link.",
            sources.display()
        ));
    }

    let canonical_sources: std::path::PathBuf = sources
        .canonicalize()
        .map_err(|error| format!("Cannot resolve '{}': {error}.", sources.display()))?;

    if !canonical_sources.starts_with(&root) {
        return Err(format!(
            "Source directory '{}' escapes the project root.",
            sources.display()
        ));
    }

    let linker: &str = manifest.linker.compiler.as_str();

    if !matches!(linker, "" | "auto" | "clang" | "gcc") {
        return Err(format!(
            "Invalid linker compiler '{linker}'. Expected auto, clang or gcc."
        ));
    }

    let compiler_arguments: Vec<String> = self::compiler_arguments(&manifest.compiler);
    let forbidden_compiler_flags: [&str; 6] =
        ["-o", "-output", "-L", "-l", "-no-executable", "-cc-args"];

    for argument in compiler_arguments {
        if forbidden_compiler_flags.contains(&argument.as_str()) {
            return Err(format!(
                "Compiler argument '{argument}' must be placed under linker.cc-args."
            ));
        }
    }

    for profile in [&manifest.profiles.dev, &manifest.profiles.release] {
        for argument in profile.compiler_args.iter() {
            if forbidden_compiler_flags.contains(&argument.as_str()) {
                return Err(format!(
                    "Profile compiler argument '{argument}' must be placed under the profile cc-args list."
                ));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variant::CompilerVariant;

    #[test]
    fn constructs_toolchain_configuration() {
        let configuration: ToolchainConfiguration = ToolchainConfiguration::new(
            semver::Version::new(0, 2, 4),
            Some(CompilerVariant::Stripped),
        );

        assert_eq!(configuration.version, semver::Version::new(0, 2, 4));
        assert_eq!(
            configuration.get_compiler_variant(),
            Some(CompilerVariant::Stripped)
        );
    }

    #[test]
    fn parses_stripped_compiler_variant() {
        let configuration: ToolchainConfiguration =
            serde_yaml::from_str("version: 0.2.4\ncompiler-variant: stripped\n").unwrap();

        assert_eq!(
            configuration.get_compiler_variant(),
            Some(CompilerVariant::Stripped)
        );
    }

    #[test]
    fn defaults_compiler_variant_to_none() {
        let configuration: ToolchainConfiguration =
            serde_yaml::from_str("version: 0.2.4\n").unwrap();

        assert_eq!(configuration.get_compiler_variant(), None);
    }

    #[test]
    fn sets_compiler_variant() {
        let mut configuration: ToolchainConfiguration =
            serde_yaml::from_str("version: 0.2.4\n").unwrap();

        configuration.set_compiler_variant(Some(CompilerVariant::Release));

        assert_eq!(
            configuration.get_compiler_variant(),
            Some(CompilerVariant::Release)
        );

        *configuration.get_compiler_variant_mut() = Some(CompilerVariant::Stripped);

        assert_eq!(
            configuration.get_compiler_variant(),
            Some(CompilerVariant::Stripped)
        );
    }
}
