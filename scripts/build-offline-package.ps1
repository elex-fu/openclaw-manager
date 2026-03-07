#!/usr/bin/env powershell
<#
.SYNOPSIS
    Cross-platform offline package build script for OpenClaw
    Downloads OpenClaw binaries from GitHub releases for all supported platforms

.DESCRIPTION
    This script downloads OpenClaw release packages from GitHub for offline installation.
    Supports all platforms: macOS (ARM64/x64), Windows (ARM64/x64), Linux (ARM64/x64)

.PARAMETER Version
    Specify version to download (default: latest)

.PARAMETER Output
    Output directory (default: ./offline-packages)

.PARAMETER Platform
    Download specific platform only (e.g., windows-x64)

.PARAMETER Help
    Show help message

.EXAMPLE
    .\build-offline-package.ps1
    Downloads all platform packages with latest version

.EXAMPLE
    .\build-offline-package.ps1 -Version v1.2.0
    Downloads specific version for all platforms

.EXAMPLE
    .\build-offline-package.ps1 -Platform windows-x64
    Downloads only Windows x64 package

.EXAMPLE
    .\build-offline-package.ps1 -Output C:\Packages -Version v1.0.0
    Downloads to custom directory
#>

[CmdletBinding()]
param(
    [Parameter()]
    [Alias("v")]
    [string]$Version = "latest",

    [Parameter()]
    [Alias("o")]
    [string]$Output = "./offline-packages",

    [Parameter()]
    [Alias("p")]
    [string]$Platform = $null,

    [Parameter()]
    [Alias("h")]
    [switch]$Help
)

# Platform configurations
$script:PLATFORMS = @(
    @{ Name = "macos-arm64"; Ext = "tar.gz"; ArchiveType = "tar" },
    @{ Name = "macos-x64"; Ext = "tar.gz"; ArchiveType = "tar" },
    @{ Name = "windows-arm64"; Ext = "zip"; ArchiveType = "zip" },
    @{ Name = "windows-x64"; Ext = "zip"; ArchiveType = "zip" },
    @{ Name = "linux-arm64"; Ext = "tar.gz"; ArchiveType = "tar" },
    @{ Name = "linux-x64"; Ext = "tar.gz"; ArchiveType = "tar" }
)

$script:GITHUB_REPO = "openclaw-ai/openclaw"
$script:DEFAULT_OUTPUT_DIR = "./offline-packages"

# Color codes for PowerShell
$script:Colors = @{
    Reset = "$([char]27)[0m"
    Bright = "$([char]27)[1m"
    Red = "$([char]27)[31m"
    Green = "$([char]27)[32m"
    Yellow = "$([char]27)[33m"
    Blue = "$([char]27)[34m"
    Cyan = "$([char]27)[36m"
}

#region Logging Functions

function Write-Info {
    param([string]$Message)
    Write-Host "$($Colors.Blue)[INFO]$($Colors.Reset) $Message"
}

function Write-Success {
    param([string]$Message)
    Write-Host "$($Colors.Green)[SUCCESS]$($Colors.Reset) $Message"
}

function Write-Warning {
    param([string]$Message)
    Write-Host "$($Colors.Yellow)[WARN]$($Colors.Reset) $Message"
}

function Write-Error {
    param([string]$Message)
    Write-Host "$($Colors.Red)[ERROR]$($Colors.Reset) $Message"
}

function Write-Progress {
    param(
        [string]$Platform,
        [string]$Message
    )
    Write-Host "$($Colors.Cyan)[$Platform]$($Colors.Reset) $Message"
}

function Write-Header {
    param([string]$Message)
    Write-Host ""
    Write-Host "$($Colors.Bright)$($Colors.Cyan)$Message$($Colors.Reset)"
    Write-Host ""
}

#endregion

#region Helper Functions

function Show-Help {
    Write-Host @"

$($Colors.Bright)OpenClaw Offline Package Builder$($Colors.Reset)

Downloads OpenClaw binaries from GitHub releases for offline installation.

$($Colors.Bright)Usage:$($Colors.Reset)
  .\build-offline-package.ps1 [options]

$($Colors.Bright)Parameters:$($Colors.Reset)
  -Version, -v    Specify version to download (default: latest)
  -Output, -o     Output directory (default: ./offline-packages)
  -Platform, -p   Download specific platform only (e.g., windows-x64)
  -Help, -h       Show this help message

$($Colors.Bright)Supported Platforms:$($Colors.Reset)
"@
    $script:PLATFORMS | ForEach-Object { Write-Host "  $($_.Name)" }

    Write-Host @"

$($Colors.Bright)Examples:$($Colors.Reset)
  .\build-offline-package.ps1
  .\build-offline-package.ps1 -Version v1.2.0
  .\build-offline-package.ps1 -Platform windows-x64
  .\build-offline-package.ps1 -Output .\packages -Version v1.0.0
"@
}

function Ensure-Directory {
    param([string]$Path)

    if (-not (Test-Path -Path $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
        Write-Info "Created directory: $Path"
    }
}

function Get-DownloadUrl {
    param(
        [hashtable]$Platform,
        [string]$Version
    )

    $versionTag = if ($Version -eq "latest") { "latest" } else { $Version }
    $filename = "openclaw-$($Platform.Name).$($Platform.Ext)"

    if ($Version -eq "latest") {
        return "https://github.com/$script:GITHUB_REPO/releases/latest/download/$filename"
    }
    return "https://github.com/$script:GITHUB_REPO/releases/download/$versionTag/$filename"
}

function Format-FileSize {
    param([long]$Size)

    if ($Size -ge 1GB) {
        return "{0:N2} GB" -f ($Size / 1GB)
    }
    elseif ($Size -ge 1MB) {
        return "{0:N2} MB" -f ($Size / 1MB)
    }
    elseif ($Size -ge 1KB) {
        return "{0:N2} KB" -f ($Size / 1KB)
    }
    else {
        return "$Size B"
    }
}

#endregion

#region Download Functions

function Download-File {
    param(
        [string]$Url,
        [string]$OutputPath,
        [hashtable]$Platform
    )

    try {
        $request = [System.Net.HttpWebRequest]::Create($Url)
        $request.UserAgent = "OpenClaw-Manager"
        $request.AllowAutoRedirect = $true
        $request.Timeout = 30000

        $response = $request.GetResponse()
        $totalSize = $response.ContentLength
        $stream = $response.GetResponseStream()

        $fileStream = [System.IO.File]::Create($OutputPath)
        $buffer = New-Object byte[] 8192
        $downloaded = 0
        $lastProgress = 0

        while ($true) {
            $read = $stream.Read($buffer, 0, $buffer.Length)
            if ($read -le 0) { break }

            $fileStream.Write($buffer, 0, $read)
            $downloaded += $read

            if ($totalSize -gt 0) {
                $progress = [math]::Round(($downloaded / $totalSize) * 100)
                if ($progress - $lastProgress -ge 10) {
                    Write-Progress -Platform $Platform.Name -Message "Download progress: $progress%"
                    $lastProgress = $progress
                }
            }
        }

        $fileStream.Close()
        $stream.Close()
        $response.Close()

        return $true
    }
    catch {
        # Clean up failed download
        if (Test-Path -Path $OutputPath) {
            Remove-Item -Path $OutputPath -Force -ErrorAction SilentlyContinue
        }
        throw $_
    }
}

function Download-Platform {
    param(
        [hashtable]$Platform,
        [string]$Version,
        [string]$OutputDir
    )

    $url = Get-DownloadUrl -Platform $Platform -Version $Version
    $outputPath = Join-Path -Path $OutputDir -ChildPath "openclaw-$($Platform.Name).$($Platform.Ext)"

    Write-Progress -Platform $Platform.Name -Message "Starting download from: $url"

    try {
        Download-File -Url $url -OutputPath $outputPath -Platform $Platform

        # Verify file
        $fileInfo = Get-Item -Path $outputPath
        if ($fileInfo.Length -eq 0) {
            throw "Downloaded file is empty"
        }

        $sizeFormatted = Format-FileSize -Size $fileInfo.Length
        Write-Success "$($Platform.Name): Downloaded $sizeFormatted"

        return @{
            Platform = $Platform.Name
            Path = $outputPath
            Size = $fileInfo.Length
            Success = $true
        }
    }
    catch {
        Write-Error "$($Platform.Name): $($_.Exception.Message)"

        # Clean up
        if (Test-Path -Path $outputPath) {
            Remove-Item -Path $outputPath -Force -ErrorAction SilentlyContinue
        }

        return @{
            Platform = $Platform.Name
            Error = $_.Exception.Message
            Success = $false
        }
    }
}

#endregion

#region Metadata Functions

function Create-Metadata {
    param(
        [string]$OutputDir,
        [string]$Version,
        [array]$Results
    )

    $successfulPackages = $Results | Where-Object { $_.Success }
    $failedPackages = $Results | Where-Object { -not $_.Success }

    $metadata = @{
        version = $Version
        createdAt = (Get-Date -Format "o")
        githubRepo = $script:GITHUB_REPO
        packages = $successfulPackages | ForEach-Object {
            @{
                platform = $_.Platform
                filename = Split-Path -Path $_.Path -Leaf
                size = $_.Size
                sizeFormatted = Format-FileSize -Size $_.Size
            }
        }
        failed = $failedPackages | ForEach-Object { $_.Platform }
    }

    $metadataPath = Join-Path -Path $OutputDir -ChildPath "metadata.json"
    $metadata | ConvertTo-Json -Depth 10 | Out-File -FilePath $metadataPath -Encoding UTF8
    Write-Info "Metadata saved to: $metadataPath"

    return $metadata
}

#endregion

#region Main Function

function Main {
    # Show help if requested
    if ($Help) {
        Show-Help
        exit 0
    }

    Write-Header "OpenClaw Offline Package Builder"
    Write-Info "Version: $Version"
    Write-Info "Output directory: $Output"

    # Ensure output directory exists
    Ensure-Directory -Path $Output

    # Filter platforms if specific one requested
    $platformsToDownload = if ($Platform) {
        $script:PLATFORMS | Where-Object { $_.Name -eq $Platform }
    }
    else {
        $script:PLATFORMS
    }

    if (-not $platformsToDownload) {
        Write-Error "Unknown platform: $Platform"
        Write-Info "Supported platforms: $($script:PLATFORMS.Name -join ', ')"
        exit 1
    }

    Write-Info "Platforms to download: $($platformsToDownload.Name -join ', ')"
    Write-Host ""

    # Download all platforms
    $results = @()
    foreach ($platform in $platformsToDownload) {
        $result = Download-Platform -Platform $platform -Version $Version -OutputDir $Output
        $results += $result
    }

    Write-Host ""
    Write-Header "Build Summary"

    # Create metadata
    $metadata = Create-Metadata -OutputDir $Output -Version $Version -Results $results

    # Print summary
    $successCount = ($results | Where-Object { $_.Success }).Count
    $failCount = ($results | Where-Object { -not $_.Success }).Count

    Write-Info "Total packages: $($results.Count)"
    Write-Success "Successful: $successCount"

    if ($failCount -gt 0) {
        Write-Error "Failed: $failCount"
        $results | Where-Object { -not $_.Success } | ForEach-Object {
            Write-Error "  - $($_.Platform): $($_.Error)"
        }
    }

    $totalSize = ($results | Where-Object { $_.Success } | Measure-Object -Property Size -Sum).Sum
    Write-Info "Total size: $(Format-FileSize -Size $totalSize)"

    if ($successCount -eq 0) {
        Write-Error "No packages were downloaded successfully"
        exit 1
    }

    $resolvedPath = Resolve-Path -Path $Output
    Write-Success "`nOffline packages built successfully in: $resolvedPath"
}

#endregion

# Execute main function
try {
    Main
}
catch {
    Write-Error "Unexpected error: $($_.Exception.Message)"
    Write-Error $_.ScriptStackTrace
    exit 1
}
