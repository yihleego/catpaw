@echo off
setlocal

REM Configuration variables
set APP_NAME=CatPaw
set CARGO_BINARY_NAME=cat-paw

echo [INFO] Compiling Release version...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Compilation failed.
    exit /b %ERRORLEVEL%
)

REM Prepare directory structure
set DIST_DIR=target\release\distribution\%APP_NAME%

echo [INFO] Creating Distribution structure: %DIST_DIR%
if exist "%DIST_DIR%" rd /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"

REM 1. Copy and rename binary file
echo [INFO] Copying executable...
set SRC_BIN=target\release\%CARGO_BINARY_NAME%.exe

if not exist "%SRC_BIN%" (
    REM Try underscore version if hyphen version doesn't exist
    set SRC_BIN=target\release\cat_paw.exe
)

if not exist "%SRC_BIN%" (
    echo [ERROR] Executable not found: %SRC_BIN%
    echo         Checked both "cat-paw.exe" and "cat_paw.exe"
    exit /b 1
)

copy "%SRC_BIN%" "%DIST_DIR%\%APP_NAME%.exe" >nul

REM 2. Copy Assets directory (Bevy resources)
if exist "assets" (
    echo [INFO] Copying assets...
    xcopy "assets" "%DIST_DIR%\assets\" /E /I /Y >nul
)

echo [SUCCESS] Build successful!
echo [INFO] Application located at: %DIST_DIR%
echo [INFO] Run "%DIST_DIR%\%APP_NAME%.exe" to launch.

endlocal

pause