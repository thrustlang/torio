@echo off
rem Copyright (C) 2026  Stevens Benavides
rem
rem This program is free software: you can redistribute it and/or modify
rem it under the terms of the GNU General Public License as published by
rem the Free Software Foundation, either version 3 of the License, or
rem (at your option) any later version.

setlocal EnableExtensions EnableDelayedExpansion
cd /d "%~dp0\.."

where git >nul 2>&1 || (echo Error: Required command 'git' was not found. & exit /b 1)
where git-cliff >nul 2>&1 || (echo Error: Required command 'git-cliff' was not found. & exit /b 1)
where cargo >nul 2>&1 || (echo Error: Required command 'cargo' was not found. & exit /b 1)
where powershell >nul 2>&1 || (echo Error: Required command 'powershell' was not found. & exit /b 1)

for /f %%i in ('git status --porcelain') do (
    echo Error: The working tree must be clean before preparing a release.
    exit /b 1
)

git fetch --tags origin || exit /b 1

echo Available tags:
git tag --sort=-version:refname
echo.

set /p prev_tag="Enter previous tag: "

if "%prev_tag%"=="" (
    echo Error: Previous tag is required.
    exit /b 1
)

git rev-parse --verify --quiet "refs/tags/%prev_tag%" >nul 2>&1
if errorlevel 1 (
    echo Error: Tag '%prev_tag%' does not exist.
    exit /b 1
)

set /p tag_name="Enter new Torio tag: "

set "release_data="
set "channel="
for /f "delims=" %%i in ('powershell -NoProfile -Command "$m=[regex]::Match($env:tag_name, '^torio-(x86_64-linux-ubuntu|x86_64-windows-msvc|x86_64-macos|aarch64-macos)(-dev)?-v(\d+\.\d+\.\d+)$'); if ($m.Success) { $m.Groups[3].Value + '|' + $m.Groups[2].Value }"') do set "release_data=%%i"

if "%release_data%"=="" (
    echo Error: Tag '%tag_name%' does not match a supported Torio stable or development tag.
    exit /b 1
)

for /f "tokens=1,2 delims=|" %%a in ("%release_data%") do (
    set "version=%%a"
    set "channel=%%b"
)

set "range=%prev_tag%..HEAD"
set "release_dir=changelogs\v%version%"
set "tag_1=torio-x86_64-linux-ubuntu%channel%-v%version%"
set "tag_2=torio-x86_64-windows-msvc%channel%-v%version%"
set "tag_3=torio-x86_64-macos%channel%-v%version%"
set "tag_4=torio-aarch64-macos%channel%-v%version%"

for %%t in ("%tag_1%" "%tag_2%" "%tag_3%" "%tag_4%") do (
    git rev-parse --verify --quiet "refs/tags/%%~t" >nul 2>&1
    if not errorlevel 1 (
        echo Error: Local tag '%%~t' already exists.
        exit /b 1
    )

    git ls-remote --exit-code --tags origin "refs/tags/%%~t" >nul 2>&1
    if not errorlevel 1 (
        echo Error: Remote tag '%%~t' already exists.
        exit /b 1
    )
)

if not exist "%release_dir%" mkdir "%release_dir%"

echo Generating changelog for: %range%
git-cliff "%range%" --tag "v%version%" --output "%release_dir%\README.md" || exit /b 1

cargo build --quiet || exit /b 1

>> "%release_dir%\README.md" echo.
>> "%release_dir%\README.md" echo ## Command Line
>> "%release_dir%\README.md" echo ```console
target\debug\torio.exe --help >> "%release_dir%\README.md" 2>&1
>> "%release_dir%\README.md" echo ```
>> "%release_dir%\README.md" echo.
>> "%release_dir%\README.md" echo ---
>> "%release_dir%\README.md" echo *Torio Changelog*

powershell -NoProfile -Command "$version=$env:version; $inPackage=$false; $content=Get-Content 'Cargo.toml' | ForEach-Object { if ($_ -match '^\[package\]') { $inPackage=$true } elseif ($_ -match '^\[') { $inPackage=$false }; if ($inPackage -and $_ -match '^version\s*=') { 'version = \"' + $version + '\"' } else { $_ } }; Set-Content -Encoding utf8 'Cargo.toml' $content" || exit /b 1

cargo check --quiet || exit /b 1

git add "%release_dir%\README.md" Cargo.toml Cargo.lock
git commit -m "Bumping 'v%version%'" || exit /b 1

for %%t in ("%tag_1%" "%tag_2%" "%tag_3%" "%tag_4%") do (
    git tag "%%~t" || exit /b 1
)

echo Prepared tags:
echo   %tag_1%
echo   %tag_2%
echo   %tag_3%
echo   %tag_4%

set /p push_answer="Push the release? [y/N] "

if /I "%push_answer%"=="y" (
    git push origin HEAD || exit /b 1

    for %%t in ("%tag_1%" "%tag_2%" "%tag_3%" "%tag_4%") do (
        git push origin "%%~t" || exit /b 1
    )

    echo Release commit and tags pushed to origin.
) else (
    echo Push skipped. The release commit and tags remain local.
)

echo Changelog generated at %release_dir%\README.md
exit /b 0
