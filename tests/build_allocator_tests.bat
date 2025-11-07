@echo off
REM Build script for Windows - Heap Allocator Tests
REM
REM Usage:
REM   build_allocator_tests.bat         Build test executable
REM   build_allocator_tests.bat test    Build and run tests
REM   build_allocator_tests.bat clean   Clean build artifacts

echo ╔════════════════════════════════════════════════════════════╗
echo ║  Glimmer-Weave Allocator Test Builder (Windows)           ║
echo ╚════════════════════════════════════════════════════════════╝
echo.

REM Check for GCC
where gcc >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ❌ ERROR: GCC not found in PATH
    echo.
    echo Please install MinGW-w64 or MSYS2 to get GCC for Windows.
    echo.
    echo Download from:
    echo   - MSYS2: https://www.msys2.org/
    echo   - MinGW-w64: https://www.mingw-w64.org/
    echo.
    exit /b 1
)

if "%1"=="clean" goto clean
if "%1"=="test" goto test

:build
echo Building allocator tests...
echo.

gcc -Wall -Wextra -g -O2 -o test_allocator.exe test_allocator.c ..\src\native_allocator.S

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo ❌ Build failed with error code %ERRORLEVEL%
    exit /b 1
)

echo.
echo ✅ Build complete: test_allocator.exe
echo.

if "%1"=="test" goto run_tests
goto end

:run_tests
echo.
echo Running tests...
echo.
test_allocator.exe
set TEST_RESULT=%ERRORLEVEL%

echo.
if %TEST_RESULT% EQU 0 (
    echo ✅ All tests passed!
) else (
    echo ❌ Tests failed with exit code %TEST_RESULT%
)
exit /b %TEST_RESULT%

:clean
echo Cleaning build artifacts...
if exist test_allocator.exe del test_allocator.exe
echo ✅ Clean complete
goto end

:end
exit /b 0
