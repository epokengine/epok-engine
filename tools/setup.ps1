# Portable dependencies by default; optional explicit per-user project registration.
[CmdletBinding()]
param(
    # Restore missing or changed distribution files from the verified archives.
    [switch]$Repair,
    [switch]$RegisterProjectFiles,
    [string]$EditorPath = '',
    # The editor installs one package without touching unrelated tools or SDK work.
    [ValidateSet('', 'mips', 'redux', 'psxavenc', 'mkpsxiso', 'libclang', 'nugget', 'serial')]
    [string]$Dependency = ''
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
if ($env:OS -ne 'Windows_NT') { throw 'This setup script supports Windows only.' }

$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$toolsRoot = Join-Path $projectRoot '.tools'
$manifest = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'dependencies.json') -Raw | ConvertFrom-Json
if ($manifest.schema_version -ne 1) { throw 'Unsupported dependency manifest version.' }
New-Item -ItemType Directory -Force -Path $toolsRoot | Out-Null
Add-Type -AssemblyName System.IO.Compression.FileSystem

function Get-StreamHash([IO.Stream]$stream) {
    $sha = [Security.Cryptography.SHA256]::Create()
    try { return [BitConverter]::ToString($sha.ComputeHash($stream)).Replace('-', '').ToLowerInvariant() }
    finally { $sha.Dispose() }
}

function Get-PathHash([string]$path) {
    $stream = [IO.File]::OpenRead($path)
    try { return Get-StreamHash $stream }
    finally { $stream.Dispose() }
}

function Install-Archive($dependency) {
    $archive = Join-Path $toolsRoot $dependency.archive
    $destination = [IO.Path]::GetFullPath((Join-Path $toolsRoot $dependency.directory))
    if (-not $destination.StartsWith($toolsRoot + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Dependency destination must remain inside .tools.'
    }
    if (-not (Test-Path -LiteralPath $archive)) {
        Write-Host "Downloading $($dependency.name) $($dependency.version)"
        $download = $archive + '.tmp'
        Invoke-WebRequest -UseBasicParsing -Uri $dependency.url -OutFile $download
        if ((Get-PathHash $download) -ne $dependency.sha256) {
            throw "Archive checksum mismatch: $download"
        }
        Move-Item -LiteralPath $download -Destination $archive
    }
    if ((Get-PathHash $archive) -ne $dependency.sha256) {
        throw "Archive checksum mismatch: $archive. Move it aside and run setup again."
    }

    # Test-Path reports dangling junctions as existing directories on Windows.
    # Preserve the link itself, never modify or remove its (possibly shared) target.
    $existing = Get-Item -LiteralPath $destination -Force -ErrorAction SilentlyContinue
    if ($existing -and ($existing.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        if (-not $Repair) {
            throw "$destination is a directory link. Run setup.ps1 -Repair to preserve the link and install a local package."
        }
        $backupName = $existing.Name + '.link-backup-' + [Guid]::NewGuid().ToString('N')
        Rename-Item -LiteralPath $destination -NewName $backupName
        Write-Host "Preserved directory link as $backupName; installing a local $($dependency.name) package."
    }

    $fresh = -not (Test-Path -LiteralPath $destination)
    $zip = [IO.Compression.ZipFile]::OpenRead($archive)
    try {
        # Compare every distribution file, including DLLs, headers and licenses.
        # Extra local files are left untouched.
        $mismatches = @()
        foreach ($entry in $zip.Entries) {
            if (-not $entry.Name) { continue }
            $installed = [IO.Path]::GetFullPath((Join-Path $destination $entry.FullName))
            if (-not $installed.StartsWith($destination + [IO.Path]::DirectorySeparatorChar, [StringComparison]::OrdinalIgnoreCase)) {
                throw "Archive entry escapes its destination: $($entry.FullName)"
            }
            $ancestor = $installed
            while ($ancestor.Length -gt $destination.Length) {
                $item = Get-Item -LiteralPath $ancestor -Force -ErrorAction SilentlyContinue
                if ($item -and ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
                    throw "Package path crosses a directory or file link: $ancestor. Preserve that link outside the package before repairing."
                }
                $ancestor = Split-Path -Parent $ancestor
            }
            $stream = $entry.Open()
            try { $expected = Get-StreamHash $stream }
            finally { $stream.Dispose() }
            $matches = (Test-Path -LiteralPath $installed -PathType Leaf) -and
                ((Get-PathHash $installed) -eq $expected)
            if ($matches) { continue }
            if (-not ($fresh -or $Repair)) {
                $mismatches += $entry.FullName
                continue
            }
            New-Item -ItemType Directory -Force -Path (Split-Path -Parent $installed) | Out-Null
            [IO.Compression.ZipFileExtensions]::ExtractToFile($entry, $installed, $true)
            if ((Get-PathHash $installed) -ne $expected) {
                throw "Installed file checksum mismatch: $installed"
            }
        }
        if ($mismatches.Count) {
            throw "$($dependency.name) has $($mismatches.Count) missing or changed files (first: $($mismatches[0])). Run setup.ps1 -Repair to restore distribution files."
        }
    }
    finally { $zip.Dispose() }
    Write-Host "Verified $($dependency.name) $($dependency.version)"
}

# Initialize only the SDK: this pinned mirror has unrelated nested submodules
# whose paths are not all valid after mirroring from PCSX-Redux.
if (-not $Dependency -or $Dependency -eq 'nugget') {
Get-Command git -ErrorAction Stop | Out-Null
$sdk = $manifest.nugget
$sdkPath = Join-Path $projectRoot $sdk.path
if (Test-Path -LiteralPath (Join-Path $projectRoot '.git')) {
    $gitlink = & git -C $projectRoot ls-files --stage -- $sdk.path
    if ($LASTEXITCODE -ne 0 -or $gitlink -notmatch '^160000 ([0-9a-f]{40}) 0\s') {
        throw 'Nugget must be registered as a submodule in this checkout.'
    }
    if ($Matches[1] -ne $sdk.revision) { throw 'Nugget gitlink and tools/dependencies.json disagree.' }
    if (-not (Test-Path -LiteralPath (Join-Path $sdkPath '.git'))) {
        & git -C $projectRoot -c submodule.recurse=false submodule update --init -- $sdk.path
        if ($LASTEXITCODE -ne 0) { throw 'Nugget submodule initialization failed.' }
    }
}
elseif ((-not (Test-Path -LiteralPath $sdkPath)) -or
    (@(Get-ChildItem -LiteralPath $sdkPath -Force).Count -eq 0)) {
    # Source archives do not contain Git submodule content.
    & git clone --no-checkout --depth 1 $sdk.url $sdkPath
    if ($LASTEXITCODE -ne 0) { throw 'Nugget clone failed.' }
    & git -C $sdkPath fetch --depth 1 origin $sdk.revision
    if ($LASTEXITCODE -ne 0) { throw 'Nugget revision fetch failed.' }
    & git -C $sdkPath -c submodule.recurse=false checkout --detach $sdk.revision
    if ($LASTEXITCODE -ne 0) { throw 'Nugget checkout failed.' }
}
if (-not (Test-Path -LiteralPath (Join-Path $sdkPath '.git'))) {
    throw 'Existing Nugget directory has no Git metadata. Move it aside and run setup again.'
}
$actual = & git -C $sdkPath rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $actual -ne $sdk.revision) {
    throw "Nugget must be at $($sdk.revision). Preserve local work before updating the submodule."
}
$changes = & git -C $sdkPath status --porcelain --untracked-files=normal
if ($LASTEXITCODE -ne 0 -or $changes) { throw 'Nugget has local source changes. Preserve them before setup.' }
foreach ($file in @('psyqo/psyqo.mk', 'common.mk', 'third_party/EASTL/include/EASTL/array.h', 'third_party/EABase/include/Common/EABase/eabase.h')) {
    if (-not (Test-Path -LiteralPath (Join-Path $sdkPath $file))) { throw "Incomplete Nugget checkout: $file" }
}
Write-Host "Verified Nugget $actual"
}
foreach ($package in $manifest.archives) {
    if (-not $Dependency -or $package.directory -eq $Dependency) { Install-Archive $package }
}
if (-not $Dependency -or $Dependency -eq 'serial') {
    $serialPackage = Get-Content -LiteralPath (Join-Path $PSScriptRoot 'serial-dependency.json') -Raw | ConvertFrom-Json
    $serialDestination = Join-Path $toolsRoot 'serial-bundle'
    New-Item -ItemType Directory -Force -Path $serialDestination | Out-Null
    foreach ($serialFile in $serialPackage.files) {
        if ($serialFile.name -match '[/\\]' -or $serialFile.name.StartsWith('.')) { throw 'Invalid serial package filename.' }
        $serialPath = Join-Path $serialDestination $serialFile.name
        if ((Test-Path -LiteralPath $serialPath -PathType Leaf) -and ((Get-PathHash $serialPath) -eq $serialFile.sha256)) { continue }
        $serialTemporary = $serialPath + '.tmp'
        Invoke-WebRequest -UseBasicParsing -Uri ($serialPackage.base_url + $serialFile.name) -OutFile $serialTemporary
        if ((Get-PathHash $serialTemporary) -ne $serialFile.sha256) { throw "Serial component checksum mismatch: $($serialFile.name)" }
        Move-Item -LiteralPath $serialTemporary -Destination $serialPath -Force
    }
    Set-Content -LiteralPath (Join-Path $serialDestination 'SOURCE.txt') -Value "Unmodified NOTPSXSerial. Source: $($serialPackage.source). See LICENSE and THIRD_PARTY_NOTICES.txt."
    Write-Host 'Verified offline PSX serial tools.'
}
if ($RegisterProjectFiles) {
    if (-not $EditorPath) {$EditorPath=Join-Path $projectRoot 'target/release/epok-editor.exe'}
    & (Join-Path $PSScriptRoot 'register-project.ps1') -EditorPath $EditorPath
}
Write-Host 'Ready. Run cargo run --locked, then Play.'
