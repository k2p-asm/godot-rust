@echo off
REM Verifies that the Rust static library is linked into the built engine and
REM that its code actually executes at startup.
REM
REM Usage: modules\rustcore\verify_link.cmd   (after a successful scons build)
setlocal
cd /d "%~dp0..\.."

if not exist "bin\godot.windows.editor.x86_64.console.exe" (
  echo FAIL: engine binary not found, build it first with scons.
  exit /b 1
)

bin\godot.windows.editor.x86_64.console.exe --headless --quit > "%TEMP%\gcr_link_check.txt" 2>&1
findstr /C:"variant tag = 2, truthy = 1" >nul
findstr /C:"Rust packed sum = 256" "%TEMP%\gcr_link_check.txt" >nul
if errorlevel 1 (
  echo FAIL: the Rust module did not report at startup. Output follows:
  type "%TEMP%\gcr_link_check.txt"
  exit /b 1
)

echo OK: Rust core module is linked and running inside the engine binary.
