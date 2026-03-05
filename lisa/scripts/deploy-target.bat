@echo off
REM Lisa -> webOS TV target deploy (ExecutionPolicy bypass wrapper)
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0deploy-target.ps1" %*
