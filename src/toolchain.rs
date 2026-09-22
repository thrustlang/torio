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

#![allow(clippy::manual_inspect)]

use isahc::{config::RedirectPolicy, prelude::*};
use serde::{Deserialize, Serialize};
use std::io::{IsTerminal, Write};

#[derive(Debug, Deserialize)]
struct GithubRelease {
    tag_name: String,
    draft: bool,
    prerelease: bool,
    assets: Vec<GithubAsset>,
}

#[derive(Debug, Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Settings {
    active: Option<String>,
}

pub fn install(requested_version: Option<&str>) -> Result<semver::Version, String> {
    let platform: String = self::platform()?;
    let interactive: bool = std::io::stdout().is_terminal();
    let mut spinner: Option<terminal_spinners::SpinnerHandle> = if interactive {
        Some(
            terminal_spinners::SpinnerBuilder::new()
                .spinner(&terminal_spinners::DOTS)
                .text(" Checking for available Thrust toolchain releases")
                .start(),
        )
    } else {
        None
    };
    let releases: Vec<GithubRelease> =
        self::fetch_releases("thrustlang/thrustc").map_err(|error| {
            if let Some(handle) = spinner.take() {
                handle.error();
            }

            error
        })?;
    let (version, release): (semver::Version, GithubRelease) =
        self::select_toolchain_release(releases, &platform, requested_version).map_err(
            |error| {
                if let Some(handle) = spinner.take() {
                    handle.error();
                }

                error
            },
        )?;

    if let Some(handle) = spinner.take() {
        handle.done();
    }

    let root: std::path::PathBuf = self::root()?;
    let version_directory_name: String = format!("v{version}");
    let compiler_destination: std::path::PathBuf =
        root.join("compiler").join(&version_directory_name);
    let lsp_destination: std::path::PathBuf = root.join("lsp").join(&version_directory_name);

    let executable_suffix: &str = if cfg!(windows) { ".exe" } else { "" };
    let compiler_name: String = format!("thrustc{executable_suffix}");
    let stripped_name: String = format!("thrustc-stripped{executable_suffix}");
    let lsp_name: String = format!("thrustc_lsp{executable_suffix}");
    let lsp_stripped_name: String = format!("thrustc_lsp-stripped{executable_suffix}");
    let has_vsix: bool = std::fs::read_dir(&lsp_destination)
        .ok()
        .is_some_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                let name: String = entry.file_name().to_string_lossy().to_string();

                name.starts_with("thrustlang-vscode-") && name.ends_with(".vsix")
            })
        });
    let existing_complete: bool = compiler_destination.join(&compiler_name).is_file()
        && compiler_destination.join(&stripped_name).is_file()
        && lsp_destination.join(&lsp_name).is_file()
        && lsp_destination.join(&lsp_stripped_name).is_file()
        && has_vsix;

    if existing_complete {
        self::activate(&version)?;
        self::install_editors(&lsp_destination);
        self::register_current_torio()?;
        self::diagnose_c_compiler();
        println!("Toolchain v{version} is already installed and is now active.");

        return Ok(version);
    }

    if compiler_destination.exists() {
        std::fs::remove_dir_all(&compiler_destination)
            .map_err(|error| format!("Cannot replace incomplete compiler v{version}: {error}."))?;
    }

    if lsp_destination.exists() {
        std::fs::remove_dir_all(&lsp_destination)
            .map_err(|error| format!("Cannot replace incomplete LSP v{version}: {error}."))?;
    }

    std::fs::create_dir_all(&root)
        .map_err(|error| format!("Cannot create '{}': {error}.", root.display()))?;

    let process_identifier: u32 = std::process::id();
    let staging: std::path::PathBuf =
        root.join(format!(".install-v{version}-{process_identifier}"));
    let staging_compiler: std::path::PathBuf = staging.join("compiler");
    let staging_lsp: std::path::PathBuf = staging.join("lsp");

    std::fs::create_dir_all(&staging_compiler)
        .map_err(|error| format!("Cannot create installation staging directory: {error}."))?;
    std::fs::create_dir_all(&staging_lsp)
        .map_err(|error| format!("Cannot create installation staging directory: {error}."))?;

    let interactive: bool = std::io::stdout().is_terminal();
    let mut spinner: Option<terminal_spinners::SpinnerHandle> = if interactive {
        Some(
            terminal_spinners::SpinnerBuilder::new()
                .spinner(&terminal_spinners::DOTS)
                .text(format!(" Downloading Thrust toolchain v{version}"))
                .start(),
        )
    } else {
        None
    };

    for asset in release.assets.iter() {
        let destination: Option<std::path::PathBuf> =
            if asset.name == compiler_name || asset.name == stripped_name {
                Some(staging_compiler.join(&asset.name))
            } else if asset.name == lsp_name
                || asset.name == lsp_stripped_name
                || (asset.name.starts_with("thrustlang-vscode-") && asset.name.ends_with(".vsix"))
            {
                Some(staging_lsp.join(&asset.name))
            } else {
                None
            };

        if let Some(destination) = destination {
            if let Some(handle) = spinner.as_ref() {
                handle.text(format!(" Downloading {}", asset.name));
            } else {
                println!("Downloading {}...", asset.name);
            }

            if let Err(error) = self::download(&asset.browser_download_url, &destination) {
                if let Some(handle) = spinner.take() {
                    handle.error();
                }

                let _ = std::fs::remove_dir_all(&staging);
                return Err(error);
            }
        }
    }

    if let Some(handle) = spinner.take() {
        handle.done();
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let executable_paths: [std::path::PathBuf; 4] = [
            staging_compiler.join(&compiler_name),
            staging_compiler.join(&stripped_name),
            staging_lsp.join(&lsp_name),
            staging_lsp.join(&lsp_stripped_name),
        ];

        for executable_path in executable_paths {
            let permissions: std::fs::Permissions = std::fs::Permissions::from_mode(0o755);

            std::fs::set_permissions(&executable_path, permissions).map_err(|error| {
                format!(
                    "Cannot make '{}' executable: {error}.",
                    executable_path.display()
                )
            })?;
        }
    }

    let compiler_parent: std::path::PathBuf = root.join("compiler");
    let lsp_parent: std::path::PathBuf = root.join("lsp");

    std::fs::create_dir_all(&compiler_parent)
        .map_err(|error| format!("Cannot create '{}': {error}.", compiler_parent.display()))?;
    std::fs::create_dir_all(&lsp_parent)
        .map_err(|error| format!("Cannot create '{}': {error}.", lsp_parent.display()))?;
    std::fs::rename(&staging_compiler, &compiler_destination)
        .map_err(|error| format!("Cannot install compiler v{version}: {error}."))?;

    if let Err(error) = std::fs::rename(&staging_lsp, &lsp_destination) {
        let _ = std::fs::remove_dir_all(&compiler_destination);
        let _ = std::fs::remove_dir_all(&staging);

        return Err(format!("Cannot install LSP v{version}: {error}."));
    }

    let _ = std::fs::remove_dir_all(&staging);

    self::activate(&version)?;
    self::install_editors(&lsp_destination);
    self::register_current_torio()?;
    self::diagnose_c_compiler();

    println!("Installed and activated Thrust toolchain v{version}.");
    Ok(version)
}

fn select_toolchain_release(
    releases: Vec<GithubRelease>,
    platform: &str,
    requested_version: Option<&str>,
) -> Result<(semver::Version, GithubRelease), String> {
    let prefix: String = format!("thrustc-{platform}-v");
    let mut candidates: Vec<(semver::Version, GithubRelease)> = Vec::new();
    let requested_version: Option<semver::Version> = requested_version
        .map(|requested| {
            let requested: &str = requested.trim_start_matches('v');

            semver::Version::parse(requested)
                .map_err(|error| format!("Invalid toolchain version '{requested}': {error}."))
        })
        .transpose()?;

    for release in releases {
        if release.draft || release.prerelease || !release.tag_name.starts_with(&prefix) {
            continue;
        }

        let version_text: &str = release.tag_name.trim_start_matches(&prefix);
        let version: semver::Version = match semver::Version::parse(version_text) {
            Ok(version) => version,
            Err(_) => continue,
        };

        if let Some(requested) = requested_version.as_ref() {
            if requested != &version {
                continue;
            }
        }

        candidates.push((version, release));
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));

    let Some((version, release)) = candidates.into_iter().next() else {
        let requested: String =
            requested_version.map_or_else(|| "latest stable".into(), |version| version.to_string());

        return Err(format!(
            "No {platform} toolchain release was found for {requested}."
        ));
    };

    let executable_suffix: &str = if cfg!(windows) { ".exe" } else { "" };
    let compiler_name: String = format!("thrustc{executable_suffix}");
    let stripped_name: String = format!("thrustc-stripped{executable_suffix}");
    let lsp_name: String = format!("thrustc_lsp{executable_suffix}");
    let lsp_stripped_name: String = format!("thrustc_lsp-stripped{executable_suffix}");
    let required_assets: [String; 4] = [compiler_name, stripped_name, lsp_name, lsp_stripped_name];
    let mut missing_assets: Vec<String> = required_assets
        .into_iter()
        .filter(|required| !release.assets.iter().any(|asset| asset.name == *required))
        .collect();

    if !release
        .assets
        .iter()
        .any(|asset| asset.name.starts_with("thrustlang-vscode-") && asset.name.ends_with(".vsix"))
    {
        missing_assets.push("thrustlang-vscode-*.vsix".into());
    }

    if !missing_assets.is_empty() {
        return Err(format!(
            "Selected {platform} toolchain release '{}' is incomplete; missing assets: {}.",
            release.tag_name,
            missing_assets.join(", ")
        ));
    }

    Ok((version, release))
}

pub fn update() -> Result<(), String> {
    let previous: Option<semver::Version> = self::active_version()?;
    let installed: semver::Version = self::install(None)?;

    if previous.as_ref() == Some(&installed) {
        println!("The active Thrust toolchain is already up to date.");
    }

    let torio_update: Result<(), String> = self::update_torio();

    if let Err(error) = torio_update {
        eprintln!("warning: Torio self-update was skipped: {error}");
    }

    Ok(())
}

pub fn use_version(version: &str) -> Result<(), String> {
    let version: &str = version.trim_start_matches('v');
    let version: semver::Version = semver::Version::parse(version)
        .map_err(|error| format!("Invalid toolchain version '{version}': {error}."))?;
    let root: std::path::PathBuf = self::root()?;
    let directory: String = format!("v{version}");
    let compiler: std::path::PathBuf = root.join("compiler").join(&directory);
    let lsp: std::path::PathBuf = root.join("lsp").join(&directory);

    if !compiler.is_dir() || !lsp.is_dir() {
        return Err(format!(
            "Toolchain v{version} is not completely installed. Run 'torio toolchain install {version}'."
        ));
    }

    self::activate(&version)?;
    self::install_editors(&lsp);
    println!("Using Thrust toolchain v{version}.");
    Ok(())
}

pub fn remove(version: &str) -> Result<(), String> {
    let version: &str = version.trim_start_matches('v');
    let version: semver::Version = semver::Version::parse(version)
        .map_err(|error| format!("Invalid toolchain version '{version}': {error}."))?;
    let active: Option<semver::Version> = self::active_version()?;

    if active.as_ref() == Some(&version) {
        return Err(format!("Cannot remove active toolchain v{version}."));
    }

    let root: std::path::PathBuf = self::root()?;
    let directory: String = format!("v{version}");
    let compiler: std::path::PathBuf = root.join("compiler").join(&directory);
    let lsp: std::path::PathBuf = root.join("lsp").join(&directory);

    if !compiler.exists() && !lsp.exists() {
        return Err(format!(
            "Toolchain v{version} is not installed. Run 'torio toolchain install {version}'."
        ));
    }

    if compiler.exists() {
        std::fs::remove_dir_all(&compiler)
            .map_err(|error| format!("Cannot remove '{}': {error}.", compiler.display()))?;
    }

    if lsp.exists() {
        std::fs::remove_dir_all(&lsp)
            .map_err(|error| format!("Cannot remove '{}': {error}.", lsp.display()))?;
    }

    println!("Removed Thrust toolchain v{version}.");
    Ok(())
}

pub fn list() -> Result<(), String> {
    let root: std::path::PathBuf = self::root()?;
    let compiler_root: std::path::PathBuf = root.join("compiler");
    let active: Option<semver::Version> = self::active_version()?;
    let mut versions: Vec<semver::Version> = Vec::new();

    if compiler_root.is_dir() {
        let entries: std::fs::ReadDir = std::fs::read_dir(&compiler_root)
            .map_err(|error| format!("Cannot inspect '{}': {error}.", compiler_root.display()))?;

        for entry in entries {
            let entry: std::fs::DirEntry = entry.map_err(|error| error.to_string())?;
            let name: String = entry.file_name().to_string_lossy().to_string();
            let version: &str = name.trim_start_matches('v');

            if let Ok(version) = semver::Version::parse(version) {
                versions.push(version);
            }
        }
    }

    versions.sort();

    if versions.is_empty() {
        println!("No Thrust toolchains are installed.");
        return Ok(());
    }

    for version in versions {
        let marker: &str = if active.as_ref() == Some(&version) {
            "*"
        } else {
            " "
        };

        println!("{marker} v{version}");
    }

    Ok(())
}

pub fn dispatch_shim(name: &str) -> Result<i32, String> {
    let version: semver::Version = self::active_version()?.ok_or_else(|| {
        "No active Thrust toolchain is configured. Run 'torio toolchain install'.".to_string()
    })?;
    let root: std::path::PathBuf = self::root()?;
    let version_directory: String = format!("v{version}");
    let executable_suffix: &str = if cfg!(windows) { ".exe" } else { "" };
    let target: std::path::PathBuf = if name.starts_with("thrustc_lsp") {
        root.join("lsp")
            .join(&version_directory)
            .join(format!("{name}{executable_suffix}"))
    } else {
        root.join("compiler")
            .join(&version_directory)
            .join(format!("{name}{executable_suffix}"))
    };

    if !target.is_file() {
        return Err(format!(
            "Active component '{}' does not exist. Run 'torio toolchain update'.",
            target.display()
        ));
    }

    let arguments: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let status: std::process::ExitStatus = std::process::Command::new(&target)
        .args(arguments)
        .status()
        .map_err(|error| format!("Cannot execute '{}': {error}.", target.display()))?;

    Ok(status.code().unwrap_or(1))
}

pub fn compiler_path(version: &semver::Version) -> Result<std::path::PathBuf, String> {
    let root: std::path::PathBuf = self::root()?;
    let suffix: &str = if cfg!(windows) { ".exe" } else { "" };
    let path: std::path::PathBuf = root
        .join("compiler")
        .join(format!("v{version}"))
        .join(format!("thrustc{suffix}"));

    if !path.is_file() {
        let active: Option<semver::Version> = self::active_version()?;
        let suggestion: String = if active.as_ref() == Some(version) {
            "torio toolchain update".into()
        } else {
            format!("torio toolchain install {version}")
        };

        return Err(format!(
            "Compiler v{version} is not installed. Run '{suggestion}'."
        ));
    }

    Ok(path)
}

pub fn active_version() -> Result<Option<semver::Version>, String> {
    let settings: Settings = self::read_settings()?;

    match settings.active {
        Some(version) => {
            let version: semver::Version = semver::Version::parse(&version)
                .map_err(|error| format!("Invalid active toolchain version: {error}."))?;

            Ok(Some(version))
        }
        None => Ok(None),
    }
}

fn update_torio() -> Result<(), String> {
    let platform: String = self::platform()?;
    let releases: Vec<GithubRelease> = self::fetch_releases("thrustlang/torio")?;
    let prefix: String = format!("torio-{platform}-v");
    let mut candidates: Vec<(semver::Version, GithubRelease)> = Vec::new();

    for release in releases {
        if release.draft || release.prerelease || !release.tag_name.starts_with(&prefix) {
            continue;
        }

        let version_text: &str = release.tag_name.trim_start_matches(&prefix);

        if let Ok(version) = semver::Version::parse(version_text) {
            candidates.push((version, release));
        }
    }

    candidates.sort_by(|left, right| right.0.cmp(&left.0));

    let Some((version, release)) = candidates.into_iter().next() else {
        return Err(format!("no Torio release exists for {platform}"));
    };

    let current: semver::Version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("Invalid current Torio version: {error}."))?;

    if version <= current {
        return Ok(());
    }

    let suffix: &str = if cfg!(windows) { ".exe" } else { "" };
    let asset_name: String = format!("torio{suffix}");
    let asset: &GithubAsset = release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
        .ok_or_else(|| format!("release v{version} does not contain '{asset_name}'"))?;
    let root: std::path::PathBuf = self::root()?;
    let destination_directory: std::path::PathBuf = root.join("torio").join(format!("v{version}"));
    let destination: std::path::PathBuf = destination_directory.join(&asset_name);

    std::fs::create_dir_all(&destination_directory).map_err(|error| {
        format!(
            "Cannot create '{}': {error}.",
            destination_directory.display()
        )
    })?;

    let interactive: bool = std::io::stdout().is_terminal();
    let spinner: Option<terminal_spinners::SpinnerHandle> = if interactive {
        Some(
            terminal_spinners::SpinnerBuilder::new()
                .spinner(&terminal_spinners::DOTS)
                .text(format!(" Downloading Torio v{version}"))
                .start(),
        )
    } else {
        None
    };
    let download_result: Result<(), String> =
        self::download(&asset.browser_download_url, &destination);

    if let Some(handle) = spinner {
        if download_result.is_ok() {
            handle.done();
        } else {
            handle.error();
        }
    }

    download_result?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let permissions: std::fs::Permissions = std::fs::Permissions::from_mode(0o755);

        std::fs::set_permissions(&destination, permissions).map_err(|error| {
            format!(
                "Cannot make '{}' executable: {error}.",
                destination.display()
            )
        })?;
    }

    let downloaded_version: String = self::executable_version(&destination)?;

    if downloaded_version != version.to_string() {
        return Err(format!(
            "downloaded Torio reports version {downloaded_version}, expected {version}"
        ));
    }

    self::activate_torio(&destination)?;
    println!("Updated Torio to v{version}.");
    Ok(())
}

fn activate(version: &semver::Version) -> Result<(), String> {
    let root: std::path::PathBuf = self::root()?;
    let version_directory: String = format!("v{version}");

    #[cfg(unix)]
    {
        let home: String = std::env::var("HOME")
            .map_err(|_| "The HOME environment variable is required.".to_string())?;
        let binary_directory: std::path::PathBuf =
            std::path::PathBuf::from(&home).join(".local/bin");

        std::fs::create_dir_all(&binary_directory)
            .map_err(|error| format!("Cannot create '{}': {error}.", binary_directory.display()))?;

        let links: [(std::path::PathBuf, std::path::PathBuf); 4] = [
            (
                binary_directory.join("thrustc"),
                root.join("compiler")
                    .join(&version_directory)
                    .join("thrustc"),
            ),
            (
                binary_directory.join("thrustc-stripped"),
                root.join("compiler")
                    .join(&version_directory)
                    .join("thrustc-stripped"),
            ),
            (
                binary_directory.join("thrustc_lsp"),
                root.join("lsp")
                    .join(&version_directory)
                    .join("thrustc_lsp"),
            ),
            (
                binary_directory.join("thrustc_lsp-stripped"),
                root.join("lsp")
                    .join(&version_directory)
                    .join("thrustc_lsp-stripped"),
            ),
        ];

        for (link, target) in links {
            let temporary_link: std::path::PathBuf = link.with_extension("torio-tmp");

            if temporary_link.exists() || temporary_link.is_symlink() {
                std::fs::remove_file(&temporary_link).map_err(|error| {
                    format!("Cannot replace '{}': {error}.", temporary_link.display())
                })?;
            }

            std::os::unix::fs::symlink(&target, &temporary_link).map_err(|error| {
                format!(
                    "Cannot link '{}' to '{}': {error}.",
                    temporary_link.display(),
                    target.display()
                )
            })?;

            std::fs::rename(&temporary_link, &link)
                .map_err(|error| format!("Cannot activate '{}': {error}.", link.display()))?;
        }

        let export_line: &str = "export PATH=\"$HOME/.local/bin:$PATH\"";
        let mut profiles: Vec<std::path::PathBuf> =
            vec![std::path::PathBuf::from(&home).join(".profile")];

        if cfg!(target_os = "macos") {
            profiles.push(std::path::PathBuf::from(&home).join(".zprofile"));
        }

        for profile in profiles {
            let profile_content: String = std::fs::read_to_string(&profile).unwrap_or_default();

            if !profile_content.contains(export_line) {
                let mut profile_file: std::fs::File = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&profile)
                    .map_err(|error| format!("Cannot update '{}': {error}.", profile.display()))?;

                writeln!(profile_file, "\n{export_line}")
                    .map_err(|error| format!("Cannot update '{}': {error}.", profile.display()))?;
            }
        }
    }

    #[cfg(windows)]
    {
        let local_app_data: String = std::env::var("LOCALAPPDATA")
            .map_err(|_| "The LOCALAPPDATA environment variable is required.".to_string())?;
        let binary_directory: std::path::PathBuf = std::path::PathBuf::from(local_app_data)
            .join("Thrust")
            .join("bin");

        std::fs::create_dir_all(&binary_directory)
            .map_err(|error| format!("Cannot create '{}': {error}.", binary_directory.display()))?;

        let current_torio: std::path::PathBuf =
            std::env::current_exe().map_err(|error| format!("Cannot locate Torio: {error}."))?;
        let shim_names: [&str; 4] = [
            "thrustc.exe",
            "thrustc-stripped.exe",
            "thrustc_lsp.exe",
            "thrustc_lsp-stripped.exe",
        ];

        for shim_name in shim_names {
            let shim: std::path::PathBuf = binary_directory.join(shim_name);

            std::fs::copy(&current_torio, &shim)
                .map_err(|error| format!("Cannot create shim '{}': {error}.", shim.display()))?;
        }

        let binary_directory_text: String = binary_directory.to_string_lossy().to_string();
        let script: String = format!(
            "$p=[Environment]::GetEnvironmentVariable('Path','User'); if ($null -eq $p) {{ $p='' }}; if (($p -split ';') -notcontains '{binary_directory_text}') {{ [Environment]::SetEnvironmentVariable('Path', ($p.TrimEnd(';') + ';{binary_directory_text}'), 'User') }}"
        );
        let status: std::process::ExitStatus = std::process::Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", &script])
            .status()
            .map_err(|error| format!("Cannot update the user PATH: {error}."))?;

        if !status.success() {
            return Err("PowerShell could not update the user PATH.".into());
        }
    }

    let mut settings: Settings = self::read_settings()?;

    settings.active = Some(version.to_string());

    let content: String = serde_yaml::to_string(&settings)
        .map_err(|error| format!("Cannot serialize toolchain settings: {error}."))?;
    let settings_path: std::path::PathBuf = root.join("settings.yml");
    let temporary_settings: std::path::PathBuf = root.join("settings.yml.tmp");

    std::fs::write(&temporary_settings, content)
        .map_err(|error| format!("Cannot write '{}': {error}.", temporary_settings.display()))?;

    #[cfg(windows)]
    if settings_path.exists() {
        std::fs::remove_file(&settings_path)
            .map_err(|error| format!("Cannot replace '{}': {error}.", settings_path.display()))?;
    }

    std::fs::rename(&temporary_settings, &settings_path)
        .map_err(|error| format!("Cannot activate toolchain v{version}: {error}."))?;

    Ok(())
}

fn register_current_torio() -> Result<(), String> {
    let root: std::path::PathBuf = self::root()?;
    let version: &str = env!("CARGO_PKG_VERSION");
    let suffix: &str = if cfg!(windows) { ".exe" } else { "" };
    let current: std::path::PathBuf = std::env::current_exe()
        .map_err(|error| format!("Cannot locate the running Torio executable: {error}."))?;
    let directory: std::path::PathBuf = root.join("torio").join(format!("v{version}"));
    let destination: std::path::PathBuf = directory.join(format!("torio{suffix}"));

    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("Cannot create '{}': {error}.", directory.display()))?;

    if current != destination {
        std::fs::copy(&current, &destination).map_err(|error| {
            format!(
                "Cannot install Torio in '{}': {error}.",
                destination.display()
            )
        })?;
    }

    self::activate_torio(&destination)
}

fn activate_torio(target: &std::path::Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        let home: String = std::env::var("HOME")
            .map_err(|_| "The HOME environment variable is required.".to_string())?;
        let binary_directory: std::path::PathBuf =
            std::path::PathBuf::from(home).join(".local/bin");
        let link: std::path::PathBuf = binary_directory.join("torio");

        std::fs::create_dir_all(&binary_directory)
            .map_err(|error| format!("Cannot create '{}': {error}.", binary_directory.display()))?;

        if link.exists() || link.is_symlink() {
            std::fs::remove_file(&link)
                .map_err(|error| format!("Cannot replace '{}': {error}.", link.display()))?;
        }

        std::os::unix::fs::symlink(target, &link).map_err(|error| {
            format!(
                "Cannot link '{}' to '{}': {error}.",
                link.display(),
                target.display()
            )
        })?;
    }

    #[cfg(windows)]
    {
        let local_app_data: String = std::env::var("LOCALAPPDATA")
            .map_err(|_| "The LOCALAPPDATA environment variable is required.".to_string())?;
        let binary_directory: std::path::PathBuf = std::path::PathBuf::from(local_app_data)
            .join("Thrust")
            .join("bin");
        let destination: std::path::PathBuf = binary_directory.join("torio.exe");

        std::fs::create_dir_all(&binary_directory)
            .map_err(|error| format!("Cannot create '{}': {error}.", binary_directory.display()))?;

        let current: std::path::PathBuf = std::env::current_exe()
            .map_err(|error| format!("Cannot locate the running Torio executable: {error}."))?;
        let current: std::path::PathBuf = current.canonicalize().unwrap_or(current);
        let active_destination: std::path::PathBuf = destination
            .canonicalize()
            .unwrap_or_else(|_| destination.clone());

        if current == active_destination {
            let target_version: String = self::executable_version(target)?;

            if target_version == env!("CARGO_PKG_VERSION") {
                return Ok(());
            }

            std::process::Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-Command",
                    "Start-Sleep -Seconds 1; Copy-Item -LiteralPath $env:TORIO_UPDATE_SOURCE -Destination $env:TORIO_UPDATE_DESTINATION -Force",
                ])
                .env("TORIO_UPDATE_SOURCE", target)
                .env("TORIO_UPDATE_DESTINATION", &destination)
                .spawn()
                .map_err(|error| format!("Cannot schedule Torio replacement: {error}."))?;

            return Ok(());
        }

        std::fs::copy(target, &destination).map_err(|error| {
            format!(
                "Cannot activate Torio in '{}': {error}.",
                destination.display()
            )
        })?;
    }

    Ok(())
}

fn install_editors(lsp_directory: &std::path::Path) {
    let vsix: Option<std::path::PathBuf> =
        std::fs::read_dir(lsp_directory).ok().and_then(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .find(|path| {
                    path.file_name().is_some_and(|name| {
                        name.to_string_lossy().starts_with("thrustlang-vscode-")
                    }) && path
                        .extension()
                        .is_some_and(|extension| extension == "vsix")
                })
        });

    let Some(vsix) = vsix else {
        eprintln!(
            "warning: no VS Code extension was found in '{}'.",
            lsp_directory.display()
        );
        return;
    };

    let editors: [&str; 4] = ["code", "code-insiders", "codium", "vscodium"];

    for editor in editors {
        let detected: bool = std::process::Command::new(editor)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success());

        if !detected {
            continue;
        }

        let installed: bool = std::process::Command::new(editor)
            .arg("--install-extension")
            .arg(&vsix)
            .arg("--force")
            .status()
            .is_ok_and(|status| status.success());

        if !installed {
            eprintln!("warning: {editor} could not install '{}'.", vsix.display());
        }
    }
}

fn fetch_releases(repository: &str) -> Result<Vec<GithubRelease>, String> {
    let url: String = format!("https://api.github.com/repos/{repository}/releases?per_page=100");
    let request: isahc::Request<()> = isahc::Request::get(&url)
        .header("User-Agent", format!("torio/{}", env!("CARGO_PKG_VERSION")))
        .header("Accept", "application/vnd.github+json")
        .body(())
        .map_err(|error| format!("Cannot create GitHub request: {error}."))?;
    let mut response: isahc::Response<isahc::Body> = request
        .send()
        .map_err(|error| format!("Cannot query GitHub releases: {error}."))?;

    if !response.status().is_success() {
        return Err(format!(
            "GitHub returned HTTP {} for {repository}.",
            response.status()
        ));
    }

    let content: String = response
        .text()
        .map_err(|error| format!("Cannot read GitHub response: {error}."))?;

    serde_json::from_str(&content)
        .map_err(|error| format!("Cannot decode GitHub releases for {repository}: {error}."))
}

fn download(url: &str, destination: &std::path::Path) -> Result<(), String> {
    let request: isahc::Request<()> = isahc::Request::get(url)
        .header("User-Agent", format!("torio/{}", env!("CARGO_PKG_VERSION")))
        .redirect_policy(RedirectPolicy::Limit(10))
        .body(())
        .map_err(|error| format!("Cannot create download request: {error}."))?;
    let mut response: isahc::Response<isahc::Body> = request
        .send()
        .map_err(|error| format!("Cannot download '{url}': {error}."))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download '{url}' returned HTTP {}.",
            response.status()
        ));
    }

    let mut file: std::fs::File = std::fs::File::create(destination)
        .map_err(|error| format!("Cannot create '{}': {error}.", destination.display()))?;

    std::io::copy(response.body_mut(), &mut file)
        .map_err(|error| format!("Cannot write '{}': {error}.", destination.display()))?;
    file.flush()
        .map_err(|error| format!("Cannot flush '{}': {error}.", destination.display()))?;
    Ok(())
}

fn executable_version(path: &std::path::Path) -> Result<String, String> {
    let output: std::process::Output =
        std::process::Command::new(path)
            .arg("--version")
            .output()
            .map_err(|error| format!("Cannot execute '{}': {error}.", path.display()))?;

    if !output.status.success() {
        return Err(format!("'{} --version' failed.", path.display()));
    }

    let output_text: String = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let version: String = output_text
        .split_whitespace()
        .last()
        .map_or_else(String::new, str::to_string);

    if version.is_empty() {
        return Err(format!(
            "'{} --version' returned an empty version.",
            path.display()
        ));
    }

    Ok(version.trim_start_matches('v').to_string())
}

fn read_settings() -> Result<Settings, String> {
    let root: std::path::PathBuf = self::root()?;
    let path: std::path::PathBuf = root.join("settings.yml");

    if !path.is_file() {
        return Ok(Settings::default());
    }

    let content: String = std::fs::read_to_string(&path)
        .map_err(|error| format!("Cannot read '{}': {error}.", path.display()))?;

    serde_yaml::from_str(&content)
        .map_err(|error| format!("Invalid toolchain settings '{}': {error}.", path.display()))
}

fn diagnose_c_compiler() {
    let clang: bool = std::process::Command::new("clang")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());
    let gcc: bool = std::process::Command::new("gcc")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success());

    if !clang && !gcc {
        eprintln!(
            "warning: neither Clang nor GCC was detected. thrustc requires a C compiler for linking."
        );
    }
}

fn platform() -> Result<String, String> {
    let operating_system: &str = std::env::consts::OS;
    let architecture: &str = std::env::consts::ARCH;

    match (operating_system, architecture) {
        ("linux", "x86_64") => Ok("x86_64-linux-ubuntu".into()),
        ("windows", "x86_64") => Ok("x86_64-windows-msvc".into()),
        ("macos", "x86_64") => Ok("x86_64-macos".into()),
        ("macos", "aarch64") => Ok("aarch64-macos".into()),
        _ => Err(format!(
            "Unsupported host platform '{architecture}-{operating_system}'. Published toolchains support Linux x86_64, Windows x86_64 and macOS x86_64/AArch64."
        )),
    }
}

fn root() -> Result<std::path::PathBuf, String> {
    if cfg!(windows) {
        let app_data: String = std::env::var("APPDATA")
            .map_err(|_| "The APPDATA environment variable is required.".to_string())?;

        return Ok(std::path::PathBuf::from(app_data).join(".thrustlang"));
    }

    if cfg!(unix) {
        let home: String = std::env::var("HOME")
            .map_err(|_| "The HOME environment variable is required.".to_string())?;

        return Ok(std::path::PathBuf::from(home).join(".thrustlang"));
    }

    Err("This operating system is not supported by Torio.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    fn release(version: &str, assets: &[&str]) -> GithubRelease {
        GithubRelease {
            tag_name: format!("thrustc-test-platform-v{version}"),
            draft: false,
            prerelease: false,
            assets: assets
                .iter()
                .map(|name| GithubAsset {
                    name: (*name).into(),
                    browser_download_url: format!("https://example.com/{name}"),
                })
                .collect(),
        }
    }

    fn complete_assets(vsix_version: &str) -> Vec<String> {
        let suffix: &str = if cfg!(windows) { ".exe" } else { "" };

        vec![
            format!("thrustc{suffix}"),
            format!("thrustc-stripped{suffix}"),
            format!("thrustc_lsp{suffix}"),
            format!("thrustc_lsp-stripped{suffix}"),
            format!("thrustlang-vscode-{vsix_version}.vsix"),
        ]
    }

    #[test]
    fn selects_latest_platform_tag_and_ignores_asset_versions() {
        let asset_names: Vec<String> = complete_assets("0.1.2");
        let assets: Vec<&str> = asset_names.iter().map(String::as_str).collect();
        let releases: Vec<GithubRelease> =
            vec![release("0.2.1", &assets), release("0.2.2", &assets)];

        let (version, selected): (semver::Version, GithubRelease) =
            select_toolchain_release(releases, "test-platform", None).unwrap();

        assert_eq!(version, semver::Version::new(0, 2, 2));
        assert_eq!(selected.tag_name, "thrustc-test-platform-v0.2.2");
        assert!(
            selected
                .assets
                .iter()
                .any(|asset| asset.name == "thrustlang-vscode-0.1.2.vsix")
        );
    }

    #[test]
    fn rejects_incomplete_latest_release_without_falling_back() {
        let asset_names: Vec<String> = complete_assets("0.2.1");
        let assets: Vec<&str> = asset_names.iter().map(String::as_str).collect();
        let suffix: &str = if cfg!(windows) { ".exe" } else { "" };
        let releases: Vec<GithubRelease> = vec![
            release("0.2.1", &assets),
            release(
                "0.2.2",
                &[
                    &format!("thrustc{suffix}"),
                    &format!("thrustc-stripped{suffix}"),
                ],
            ),
        ];

        let error: String = select_toolchain_release(releases, "test-platform", None).unwrap_err();

        assert!(error.contains("thrustc-test-platform-v0.2.2"));
        assert!(error.contains("thrustc_lsp"));
        assert!(error.contains("thrustlang-vscode-*.vsix"));
    }

    #[test]
    fn downloads_github_style_redirects() {
        let listener: std::net::TcpListener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address: std::net::SocketAddr = listener.local_addr().unwrap();
        let server: std::thread::JoinHandle<()> = std::thread::spawn(move || {
            for connection in listener.incoming().take(2) {
                let mut stream: std::net::TcpStream = connection.unwrap();
                let mut request: [u8; 1024] = [0; 1024];
                let bytes_read: usize = stream.read(&mut request).unwrap();
                let request: String = String::from_utf8_lossy(&request[..bytes_read]).into();
                let response: String = if request.starts_with("GET /asset ") {
                    format!(
                        "HTTP/1.1 302 Found\r\nLocation: http://{address}/payload\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    )
                } else {
                    "HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\npayload"
                        .into()
                };

                stream.write_all(response.as_bytes()).unwrap();
            }
        });
        let unique: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let destination: std::path::PathBuf =
            std::env::temp_dir().join(format!("torio-download-{unique}"));

        download(&format!("http://{address}/asset"), &destination).unwrap();
        server.join().unwrap();

        assert_eq!(std::fs::read(&destination).unwrap(), b"payload");
        std::fs::remove_file(destination).unwrap();
    }
}
