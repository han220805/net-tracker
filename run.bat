@echo off
title Net Tracker Desktop Launcher
cd /d "D:\Project\Dekstop\net-tracker"

:: Ensure Cargo, MinGW GCC, and Bun are in PATH for this session
set "PATH=C:\Users\codehero\.cargo\bin;C:\msys64\mingw64\bin;C:\Users\codehero\.bun\bin;%PATH%"

echo Starting Net Tracker Desktop Application...
bun run desktop
pause
