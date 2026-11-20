@echo off
REM Regenerates tests/fixtures/cpp_math_parity.txt from Godot's own C++ math code.
REM Usage (from anywhere): modules\rustcore\parity\run.cmd
setlocal
set ROOT=%~dp0..\..\..
set VS=%ProgramFiles%\Microsoft Visual Studio\2022\Community\Common7\Tools\VsDevCmd.bat
pushd "%ROOT%"
call "%VS%" -arch=x64 -no_logo >nul || (popd & exit /b 1)
cl /nologo /std:c++20 /Zc:__cplusplus /EHsc /O2 /I . /I platform\windows /I thirdparty\misc ^
   modules\rustcore\parity\dump.cpp ^
   core\math\vector2.cpp core\math\vector2i.cpp core\math\vector3i.cpp core\math\vector4.cpp core\math\vector4i.cpp core\math\vector3.cpp core\math\basis.cpp core\math\quaternion.cpp ^
   core\math\math_funcs.cpp core\math\aabb.cpp core\math\plane.cpp core\math\face3.cpp core\math\geometry_3d.cpp core\math\projection.cpp core\math\rect2.cpp core\math\transform_2d.cpp core\math\transform_3d.cpp ^
   core\templates\hashfuncs.cpp thirdparty\misc\pcg.cpp ^
   modules\rustcore\parity\rng_stub.cpp modules\rustcore\parity\string_stub.cpp ^
   /Fe:modules\rustcore\parity\dump.exe
if errorlevel 1 (
  echo BUILD FAILED
  popd
  exit /b 1
)
modules\rustcore\parity\dump.exe > modules\rustcore\tests\fixtures\cpp_math_parity.txt
modules\rustcore\parity\okcolor_dump.exe > modules\rustcore\tests\fixtures\cpp_okcolor_parity.txt
echo Wrote modules\rustcore\tests\fixtures\cpp_math_parity.txt
popd
