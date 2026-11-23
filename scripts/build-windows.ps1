[CmdletBinding()]
param(
    [ValidateSet("amd64", "arm64")]
    [string]$TargetCpu = "amd64",
    [string]$DistDir = ""
)

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

if (-not $DistDir) {
    $DistDir = Join-Path $root "dist"
}

# Prefer rustup's shims so cargo and rustc use the same toolchain.
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path (Join-Path $cargoBin "cargo.exe")) {
    $env:Path = "$cargoBin;$env:Path"
}

$cargo = Get-Command cargo.exe -ErrorAction SilentlyContinue
if (-not $cargo) {
    throw "Cargo was not found. Install Rust with rustup, then reopen PowerShell."
}

$target = if ($TargetCpu -eq "amd64") {
    "x86_64-pc-windows-msvc"
} else {
    "aarch64-pc-windows-msvc"
}

$rustup = Get-Command rustup.exe -ErrorAction SilentlyContinue
if ($rustup) {
    & $rustup.Source target add $target
    if ($LASTEXITCODE -ne 0) {
        throw "Could not install Rust target $target (exit code $LASTEXITCODE)."
    }
}

$versionLine = Select-String `
    -Path (Join-Path $root "Cargo.toml") `
    -Pattern '^version\s*=\s*"([^"]+)"' |
    Select-Object -First 1
if (-not $versionLine) {
    throw "Could not read the package version from Cargo.toml."
}
$version = $versionLine.Matches.Groups[1].Value

$exeName = "thinkpad-auto-scroll-$TargetCpu.exe"
$builtExe = Join-Path $root "target\$target\release\thinkpad-auto-scroll.exe"
$dist = [IO.Path]::GetFullPath($DistDir)

Push-Location $root
try {
    & $cargo.Source build --locked --release --target $target
    if ($LASTEXITCODE -ne 0) {
        throw "Windows release build failed (exit code $LASTEXITCODE)."
    }
    if (-not (Test-Path -LiteralPath $builtExe -PathType Leaf)) {
        throw "Build output was not found: $builtExe"
    }

    New-Item -ItemType Directory -Force -Path $dist | Out-Null
    $outputExe = Join-Path $dist $exeName
    Copy-Item -LiteralPath $builtExe -Destination $outputExe -Force
    Write-Host "Created Windows $TargetCpu executable (version $version): $outputExe"
} finally {
    Pop-Location
}
