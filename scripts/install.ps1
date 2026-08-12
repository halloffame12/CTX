# ctx — Windows installer
#
#   irm https://ctx.dev/install.ps1 | iex
#   $env:CTX_VERSION = "v0.2.0"; irm https://ctx.dev/install.ps1 | iex
#
# Installs the prebuilt ctx.exe into $env:LOCALAPPDATA\ctx\bin, verifies its
# SHA-256 checksum, and adds the directory to the user's PATH. No admin
# required. Will not overwrite an existing binary without warning.
$ErrorActionPreference = "Stop"

$repo = if ($env:CTX_REPO) { $env:CTX_REPO } else { "halloffame12/CTX" }
$version = if ($env:CTX_VERSION) { $env:CTX_VERSION } else { "latest" }
$installDir = if ($env:CTX_INSTALL_DIR) { $env:CTX_INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "ctx" }
$binDir = Join-Path $installDir "bin"
$baseUrl = "https://github.com/$repo/releases/download"

function Write-Info([string]$msg) { Write-Host $msg }

function Get-Arch {
  $arch = $env:PROCESSOR_ARCHITECTURE
  if (-not $arch) { $arch = "AMD64" }
  if ($arch -match "ARM64|Arm64|arm64") { return "aarch64" }
  return "x86_64"
}

function Get-RedirectedUrl([string]$url) {
  $resp = Invoke-WebRequest -Uri $url -Method Head -MaximumRedirection 0 -ErrorAction SilentlyContinue
  return $resp.Headers.Location
}

$arch = Get-Arch
$artifact = "ctx-windows-$arch.exe"
$exePath = Join-Path $binDir $artifact

if ($version -eq "latest") {
  $api = "https://api.github.com/repos/$repo/releases/latest"
  $rel = Invoke-RestMethod -Uri $api -Headers @{ "User-Agent" = "ctx-install" }
  $version = $rel.tag_name
}
if (-not $version.StartsWith("v")) { $version = "v$version" }

Write-Info "ctx — installing $version ($arch)"
Write-Info "  from: $baseUrl/$version/$artifact"

# Warn before overwriting an existing binary.
if (Test-Path $exePath) {
  $old = (& $exePath --version 2>$null | Select-Object -First 1)
  if ($old -eq "ctx $($version.TrimStart('v'))") {
    Write-Info "  ctx already installed at $exePath"
    exit 0
  }
  Write-Warning "overwriting existing binary at $exePath ($old)"
}

New-Item -ItemType Directory -Force -Path $binDir | Out-Null

$tmpDir = Join-Path $installDir (".install.tmp." + [System.Guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Force -Path $tmpDir | Out-Null
$tmpExe = Join-Path $tmpDir $artifact
$tmpChecksums = Join-Path $tmpDir "checksums.txt"

try {
  Write-Info "  downloading $artifact ..."
  Invoke-WebRequest -Uri "$baseUrl/$version/$artifact" -OutFile $tmpExe -UseBasicParsing
  Invoke-WebRequest -Uri "$baseUrl/$version/checksums.txt" -OutFile $tmpChecksums -UseBasicParsing

  $want = (Get-Content $tmpChecksums | Where-Object { $_ -match [regex]::Escape("  $artifact") } | Select-Object -First 1).Split(" ")[0]
  if (-not $want) { throw "could not find checksum for $artifact in checksums.txt" }

  $got = (Get-FileHash -Algorithm SHA256 -Path $tmpExe).Hash.ToLowerInvariant()
  if ($got -ne $want) { throw "checksum mismatch for $artifact (got $got, want $want)" }

  Move-Item -Force $tmpExe $exePath
  Write-Info "  installed to $exePath"

  # Add to the user PATH if missing.
  $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
  if ($userPath -notmatch [regex]::Escape($binDir)) {
    $newPath = if ([string]::IsNullOrEmpty($userPath)) { $binDir } else { "$userPath;$binDir" }
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Info "  added $binDir to your user PATH"
    Write-Info "  restart your terminal (or run: `$env:Path = \"$binDir;\" + `$env:Path)"
  }

  $ver = & $exePath --version 2>&1 | Select-Object -First 1
  if ($ver -ne "ctx $($version.TrimStart('v'))") { throw "post-install check failed: got '$ver'" }
  Write-Info "  `u{2713} $ver"
  Write-Info "  next: ctx init"
}
finally {
  Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue
}