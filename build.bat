@echo off
echo ================================
echo Building Oxistart...
echo ================================
echo.

cargo build --release

if %ERRORLEVEL% EQU 0 (
    echo.
    echo ================================
    echo Build successful!
    echo Executable: target\release\oxistart.exe
    echo ================================
) else (
    echo.
    echo ================================
    echo Build failed!
    echo ================================
)

echo.
pause