@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
powershell.exe -NoLogo -NoProfile -ExecutionPolicy Bypass -File "%SCRIPT_DIR%build-windows.ps1" %*
set "BUILD_EXIT_CODE=%ERRORLEVEL%"

if not "%BUILD_EXIT_CODE%"=="0" (
    echo.
    echo Windows build failed with exit code %BUILD_EXIT_CODE%.
)

if "%~1"=="" pause
exit /b %BUILD_EXIT_CODE%
