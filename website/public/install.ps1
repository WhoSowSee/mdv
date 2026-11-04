param(
    [Alias('Version')][string]$MdvVersion = 'latest',
    [Alias('InstallDir')][string]$MdvInstallDir = '',
    [Alias('NoModifyPath')][switch]$MdvNoModifyPath
)

& {
    param([string]$RequestedVersion, [string]$Destination, [bool]$SkipPath)

    Set-StrictMode -Version Latest
    $ErrorActionPreference = 'Stop'

    function Test-MdvPathEntry([string]$Value, [string]$Directory) {
        foreach ($entry in ($Value -split ';')) {
            $expanded = [Environment]::ExpandEnvironmentVariables($entry).Trim().Trim('"').TrimEnd('\', '/')
            if ($expanded -ieq $Directory.TrimEnd('\', '/')) { return $true }
        }
        return $false
    }

    function Add-MdvToPath([string]$Directory) {
        $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
        if (-not (Test-MdvPathEntry $userPath $Directory)) {
            if ([string]::IsNullOrEmpty($userPath)) {
                $updated = $Directory
            } elseif ($userPath.EndsWith(';')) {
                $updated = $userPath + $Directory
            } else {
                $updated = $userPath + ';' + $Directory
            }
            [Environment]::SetEnvironmentVariable('Path', $updated, 'User')
        }
        if (-not (Test-MdvPathEntry $env:Path $Directory)) {
            $env:Path = $Directory + ';' + $env:Path
        }
    }

    function Expand-MdvBinary([string]$ArchivePath, [string]$OutputPath) {
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $archive = [IO.Compression.ZipFile]::OpenRead($ArchivePath)
        try {
            $entries = @($archive.Entries | Where-Object { $_.FullName -ceq 'mdv.exe' })
            if ($entries.Count -ne 1 -or $entries[0].Length -eq 0) {
                throw 'Archive must contain exactly one nonempty root-level mdv.exe'
            }
            $inputStream = $entries[0].Open()
            try {
                $outputStream = [IO.File]::Open($OutputPath, [IO.FileMode]::CreateNew)
                try { $inputStream.CopyTo($outputStream) }
                finally { $outputStream.Dispose() }
            } finally { $inputStream.Dispose() }
        } finally { $archive.Dispose() }
    }

    function Get-MdvSha256([string]$ArchivePath) {
        $algorithm = [Security.Cryptography.SHA256]::Create()
        try {
            $stream = [IO.File]::OpenRead($ArchivePath)
            try { return [BitConverter]::ToString($algorithm.ComputeHash($stream)).Replace('-', '').ToLowerInvariant() }
            finally { $stream.Dispose() }
        } finally { $algorithm.Dispose() }
    }

    if ($env:OS -ne 'Windows_NT') { throw 'This installer supports Windows only' }
    if ($RequestedVersion -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._+-]*$') {
        throw 'Invalid release tag'
    }
    $architecture = [Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    switch ($architecture) {
        'X64' { $target = 'x86_64-pc-windows-msvc' }
        'Arm64' { $target = 'aarch64-pc-windows-msvc' }
        default { throw "Unsupported Windows architecture: $architecture" }
    }
    if ([string]::IsNullOrEmpty($Destination)) {
        $Destination = Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'Programs\mdv'
    }
    $Destination = [IO.Path]::GetFullPath($Destination)
    $binaryPath = Join-Path $Destination 'mdv.exe'
    if (Test-Path -LiteralPath $binaryPath -PathType Container) {
        throw "Destination is a directory: $binaryPath"
    }

    $api = 'https://api.github.com/repos/WhoSowSee/mdv/releases'
    if ($RequestedVersion -eq 'latest') { $releaseUrl = "$api/latest" }
    else { $releaseUrl = "$api/tags/$RequestedVersion" }
    $headers = @{ Accept = 'application/vnd.github+json'; 'User-Agent' = 'mdv-installer' }
    $release = Invoke-RestMethod -Uri $releaseUrl -Headers $headers -UseBasicParsing
    $tag = $release.tag_name
    if ($tag -cnotmatch '^[A-Za-z0-9][A-Za-z0-9._+-]*$') {
        throw 'Invalid release tag in GitHub response'
    }
    $assetName = "mdv-$tag-$target.zip"
    $assets = @($release.assets | Where-Object { $_.name -ceq $assetName })
    if ($assets.Count -ne 1) { throw "Missing unique release asset: $assetName" }
    if (-not $assets[0].PSObject.Properties['digest'] -or $assets[0].digest -cnotmatch '^sha256:[a-f0-9]{64}$') {
        throw "Missing valid SHA-256 digest: $assetName"
    }
    $expected = $assets[0].digest.Substring(7)

    $temporaryRoot = [IO.Path]::GetFullPath([IO.Path]::GetTempPath())
    $workDirectory = Join-Path $temporaryRoot ('mdv-install-' + [Guid]::NewGuid().ToString('N'))
    New-Item -ItemType Directory -Path $workDirectory | Out-Null
    $stagedPath = $null
    try {
        $archivePath = Join-Path $workDirectory 'mdv.zip'
        Write-Host "Downloading $assetName..."
        Invoke-WebRequest -Uri "https://github.com/WhoSowSee/mdv/releases/download/$tag/$assetName" -OutFile $archivePath -Headers $headers -UseBasicParsing
        $actual = Get-MdvSha256 $archivePath
        if ($actual -ine $expected) {
            throw 'SHA-256 mismatch; existing installation was not changed'
        }
        $extractedPath = Join-Path $workDirectory 'mdv.exe'
        Expand-MdvBinary $archivePath $extractedPath
        New-Item -ItemType Directory -Path $Destination -Force | Out-Null
        $stagedPath = Join-Path $Destination ('.mdv-' + [Guid]::NewGuid().ToString('N') + '.tmp')
        [IO.File]::Copy($extractedPath, $stagedPath, $false)
        if ([IO.File]::Exists($binaryPath)) {
            [IO.File]::Replace($stagedPath, $binaryPath, [NullString]::Value)
        } else {
            [IO.File]::Move($stagedPath, $binaryPath)
        }
        if (-not $SkipPath) { Add-MdvToPath $Destination }
        Write-Host "Installed mdv $($tag.TrimStart('v')) at $binaryPath"
        if ($SkipPath) { Write-Host "Add the installation directory to PATH if needed: $Destination" }
    } finally {
        if ($stagedPath -and [IO.File]::Exists($stagedPath)) { [IO.File]::Delete($stagedPath) }
        if ([IO.Directory]::Exists($workDirectory)) {
            if ([IO.Path]::GetDirectoryName($workDirectory).TrimEnd('\', '/') -ine $temporaryRoot.TrimEnd('\', '/') -or
                [IO.Path]::GetFileName($workDirectory) -notlike 'mdv-install-*') {
                throw 'Refusing to remove an unexpected temporary directory'
            }
            Remove-Item -LiteralPath $workDirectory -Recurse -Force
        }
    }
} $MdvVersion $MdvInstallDir $MdvNoModifyPath.IsPresent
