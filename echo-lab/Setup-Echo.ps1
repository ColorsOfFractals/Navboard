$ErrorActionPreference = 'Stop'

$lock = Join-Path $PSScriptRoot 'requirements-win-x64.lock'
$base = Join-Path $env:LOCALAPPDATA 'Navboard\Echo'
$runtime = Join-Path $base 'venv'
$python = Join-Path $runtime 'Scripts\python.exe'

if (!(Test-Path -LiteralPath $lock)) {
    throw 'Echo dependency lock is missing.'
}

Write-Host 'Echo requires Python 3.13 x64 from python.org.'
Write-Host 'Setup downloads hash-locked dependencies from PyPI.'
Write-Host 'The speech recognition model downloads separately on first use.'

if ((Read-Host 'Continue? Type YES') -cne 'YES') {
    return
}

if (!(Test-Path -LiteralPath $python)) {
    $launcher = Get-Command py.exe -ErrorAction SilentlyContinue

    if (!$launcher) {
        throw 'Install Python 3.13 x64 from https://www.python.org/downloads/windows/ with the Python launcher enabled.'
    }

    & $launcher.Source -3.13 -I -c 'import sys,struct; assert sys.version_info[:2]==(3,13) and struct.calcsize(''P'')==8'

    if ($LASTEXITCODE -ne 0) {
        throw 'Python 3.13 x64 is required.'
    }

    New-Item -ItemType Directory -Path $base -Force |
        Out-Null

    & $launcher.Source -3.13 -I -m venv $runtime

    if ($LASTEXITCODE -ne 0) {
        throw 'Echo environment creation failed.'
    }
}

& $python -I -m pip --isolated install `
    --disable-pip-version-check `
    --index-url https://pypi.org/simple `
    --only-binary=:all: `
    --require-hashes `
    --requirement $lock

if ($LASTEXITCODE -ne 0) {
    throw 'Dependency installation failed. Retry setup.'
}

& $python -I -m pip check

if ($LASTEXITCODE -ne 0) {
    throw 'Dependency consistency check failed.'
}

& $python -I -c 'import faster_whisper, sounddevice, soundcard, av, numpy, scipy; print(''Echo runtime healthy'')'

if ($LASTEXITCODE -ne 0) {
    throw 'Echo runtime validation failed.'
}

Write-Host 'Echo setup complete. Restart Navboard.' -ForegroundColor Green