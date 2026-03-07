@echo off
setlocal
set "DIR=%~dp0"
cd /d "%DIR%"
set "NODE_PATH=%DIR%\node_modules"
if exist "dist\index.js" (
    node dist\index.js %*
) else (
    echo Error: OpenClaw not built.
    exit /b 1
)
