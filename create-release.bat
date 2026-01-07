@echo off
REM Script to create a GitHub release with binaries
REM Usage: create-release.bat <version> <release-notes>
REM Example: create-release.bat v0.1.0 "Initial release"

if "%1"=="" (
    echo Usage: create-release.bat ^<version^> ^<release-notes^>
    echo Example: create-release.bat v0.1.0 "Initial release"
    exit /b 1
)

set VERSION=%1
set NOTES=%2

if "%NOTES%"=="" set NOTES=%VERSION%

echo Building release binaries...
call cargo build --release

echo Creating tag %VERSION%...
git tag -a %VERSION% -m "%NOTES%"
git push origin %VERSION%

echo.
echo Tag created and pushed!
echo Now go to GitHub and create a release from this tag, or use GitHub CLI:
echo gh release create %VERSION% --title "%VERSION%" --notes "%NOTES%"
echo.
echo To attach binaries, you can add them manually in the GitHub UI or use:
echo gh release upload %VERSION% target\release\pseudocode.exe


