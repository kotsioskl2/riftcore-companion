$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootDir = Split-Path -Parent $scriptDir

# 1. Read current version from package.json
$pkgJsonPath = Join-Path $rootDir "package.json"
$pkg = Get-Content $pkgJsonPath -Raw | ConvertFrom-Json
$version = $pkg.version

Write-Host "===================================================" -ForegroundColor Cyan
Write-Host "  RiftCore Companion - GitHub Release Publisher" -ForegroundColor Cyan
Write-Host "  Target Release: v$version" -ForegroundColor Cyan
Write-Host "===================================================" -ForegroundColor Cyan
Write-Host ""

# 2. Extract GitHub credentials
Write-Host "Resolving GitHub credentials from git..." -ForegroundColor Gray
$p = "protocol=https`nhost=github.com`n`n"
$output = $p | git credential fill

$token = $null
foreach ($line in ($output -split "`r?`n")) {
    if ($line -match "^password=(.+)$") {
        $token = $matches[1].Trim()
        break
    }
}

if (-not $token) {
    Write-Error "Could not retrieve GitHub credentials via git."
    exit 1
}

Write-Host "Found GitHub token for authenticated publishing." -ForegroundColor Green

$headers = @{
    "Authorization" = "Bearer $token"
    "Accept" = "application/vnd.github+json"
    "User-Agent" = "RiftCore-ReleaseScript"
}

# 3. Read update notes
$notesPath = Join-Path $rootDir "installer\UPDATE_NOTES.txt"
$updateNotes = ""
if (Test-Path $notesPath) {
    $updateNotes = [System.IO.File]::ReadAllText($notesPath)
} else {
    $updateNotes = "RiftCore Companion Release v$version"
}

# 4. Check or create GitHub Release
$tagName = "v$version"
$release = $null
try {
    $existing = Invoke-RestMethod -Uri "https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/tags/$tagName" -Method Get -Headers $headers
    $release = $existing
    Write-Host "Found existing release for $tagName (ID: $($release.id))" -ForegroundColor Yellow
} catch {
    Write-Host "Release for $tagName does not exist yet. Creating..." -ForegroundColor Cyan
}

if (-not $release) {
    $payload = @{
        tag_name = $tagName
        target_commitish = "main"
        name = "RiftCore Companion $tagName"
        body = $updateNotes
        draft = $false
        prerelease = $false
    } | ConvertTo-Json

    $release = Invoke-RestMethod -Uri "https://api.github.com/repos/kotsioskl2/riftcore-companion/releases" -Method Post -Headers $headers -Body $payload -ContentType "application/json"
    Write-Host "Created Release $tagName (ID: $($release.id))!" -ForegroundColor Green
}

$rawUploadUrl = $release.upload_url
$uploadUrlBase = $rawUploadUrl -replace '\{\?name,label\}', ''

# 5. Asset Upload Helper
function Upload-Asset($filePath, $contentType) {
    if (-not (Test-Path $filePath)) {
        Write-Warning "Asset file not found: $filePath. Skipping."
        return
    }

    $fileName = [System.IO.Path]::GetFileName($filePath)
    Write-Host "Uploading $fileName..." -ForegroundColor Cyan

    # Refresh release info to check existing assets
    $currentRelease = Invoke-RestMethod -Uri "https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/$($release.id)" -Method Get -Headers $headers
    if ($currentRelease.assets) {
        foreach ($asset in $currentRelease.assets) {
            if ($asset.name -eq $fileName) {
                Write-Host "  Replacing existing asset ID $($asset.id)..." -ForegroundColor Gray
                $delUri = "https://api.github.com/repos/kotsioskl2/riftcore-companion/releases/assets/$($asset.id)"
                Invoke-RestMethod -Uri $delUri -Method Delete -Headers $headers
                break
            }
        }
    }

    $uploadUri = "$uploadUrlBase`?name=$fileName"
    $fileBytes = [System.IO.File]::ReadAllBytes($filePath)
    
    $uploadHeaders = @{
        "Authorization" = "Bearer $token"
        "Accept" = "application/vnd.github+json"
        "User-Agent" = "RiftCore-ReleaseScript"
        "Content-Type" = $contentType
    }

    $res = Invoke-RestMethod -Uri $uploadUri -Method Post -Headers $uploadHeaders -Body $fileBytes
    Write-Host "  [OK] Uploaded $fileName! ($([math]::Round($fileBytes.Length / 1MB, 2)) MB)" -ForegroundColor Green
}

# Upload all 3 release artifacts
Upload-Asset (Join-Path $rootDir "RiftCore_Rust_Portable\RiftCore.exe") "application/vnd.microsoft.portable-executable"
Upload-Asset (Join-Path $rootDir "installer_output\RiftCore-Setup-$version.exe") "application/vnd.microsoft.portable-executable"
Upload-Asset (Join-Path $rootDir "installer_output\RiftCore-Portable-$version.zip") "application/zip"

Write-Host ""
Write-Host "===================================================" -ForegroundColor Green
Write-Host "  RELEASE PUBLISHED SUCCESSFULLY TO GITHUB!" -ForegroundColor Green
Write-Host "  URL: https://github.com/kotsioskl2/riftcore-companion/releases/tag/$tagName" -ForegroundColor Green
Write-Host "===================================================" -ForegroundColor Green
