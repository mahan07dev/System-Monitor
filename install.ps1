# install.ps1 - Windows Installer for System Monitor
$ErrorActionPreference = "Stop"

$repo = "mahan07dev/System-Monitor"
$appName = "System Monitor"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " Installing $appName for Windows..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# 1. Fetch Latest Release Details from GitHub
$releaseUrl = "https://api.github.com/repos/$repo/releases/latest"
$release = Invoke-RestMethod -Uri $releaseUrl -Headers @{ "User-Agent" = "PowerShell" }

# 2. Find the EXE installer URL (System.Monitor_1.0.1_x64-setup.exe)
$asset = $release.assets | Where-Object { $_.name -like "*-setup.exe" -or $_.name -like "*.exe" } | Select-Object -First 1

if (-not $asset) {
    Write-Error "❌ Could not find a valid .exe installer in the latest release."
    exit 1
}

$downloadUrl = $asset.browser_download_url
$fileName = $asset.name
$expectedSha256 = "f1a9ddec33e2fc538b388ee6a19f996445720c4075cd43ba5c8d7c4fc1b7c63b"
$tempPath = Join-Path $env:TEMP $fileName

# 3. Download the Installer
Write-Host "--> Downloading $fileName..." -ForegroundColor Yellow
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempPath

# 4. Verify SHA-256 Checksum
Write-Host "--> Verifying SHA-256 Checksum..." -ForegroundColor Yellow
$actualSha256 = (Get-FileHash -Path $tempPath -Algorithm SHA256).Hash.ToLower()

if ($actualSha256 -ne $expectedSha256.ToLower()) {
    Write-Error "❌ SHA-256 Checksum mismatch!`nExpected: $expectedSha256`nGot:      $actualSha256"
    Remove-Item $tempPath -ErrorAction SilentlyContinue
    exit 1
}
Write-Host "--> SHA-256 Verified successfully!" -ForegroundColor Green

# 5. Run the NSIS/Tauri Installer silently or interactively
Write-Host "--> Launching installer..." -ForegroundColor Yellow
Start-Process -FilePath $tempPath -Wait

# Cleanup
Remove-Item $tempPath -ErrorAction SilentlyContinue

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host " 🎉 $appName installation complete!" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Cyan