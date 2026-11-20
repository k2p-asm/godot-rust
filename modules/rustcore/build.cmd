@echo off
REM Builds the Rust crate inside a Visual Studio environment.
REM Usage: modules\rustcore\build.cmd [--release]
setlocal
set VS=%ProgramFiles%\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat
call "%VS%" -arch=x64 -no_logo >nul || exit /b 1
cd /d "%~dp0..\.."
if "%~1"=="--release" (
  cargo build --release --manifest-path modules\rustcore/Cargo.toml
) else (
  cargo build --manifest-path modules\rustcore/Cargo.toml
)
