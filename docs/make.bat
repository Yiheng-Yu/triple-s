@ECHO OFF
setlocal
if "%SPHINXBUILD%" == "" set SPHINXBUILD=sphinx-build
if "%BUILDDIR%" == "" set "BUILDDIR=%~dp0_build"
set "TARGET=%~1"
if "%TARGET%" == "" set TARGET=help
%SPHINXBUILD% -M %TARGET% "%~dp0." "%BUILDDIR%" %SPHINXOPTS% %O%
exit /b %ERRORLEVEL%
