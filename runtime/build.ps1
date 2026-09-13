# Standalone export launcher. No editor or host reflection extractor is required.
[CmdletBinding()]
param(
    [string]$Make = 'make.exe',
    [string]$Nugget = 'third_party/nugget',
    [string]$ToolchainBin = '',
    [string]$Build = 'Release'
)
$ErrorActionPreference = 'Stop'
$buildFolder = $PSScriptRoot
if ($buildFolder -match '[^\x00-\x7F]') {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class EpokBuildPath {
    [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
    public static extern uint GetShortPathName(string path, StringBuilder output, uint capacity);
}
'@
    $shortName = New-Object System.Text.StringBuilder 32768
    if ([EpokBuildPath]::GetShortPathName($buildFolder, $shortName, 32768) -eq 0 -or $shortName.ToString() -match '[^\x00-\x7F]') {
        throw 'GNU Make requires an ASCII working-directory alias. Windows short names are unavailable; build the complete export from an ASCII folder.'
    }
    $buildFolder = $shortName.ToString()
}
# GNU binutils cannot create relative outputs from an extended-length CWD.
# Short aliases fit the ordinary drive form; Rust's process launcher does the
# equivalent normalization for editor builds.
if ($buildFolder.StartsWith('\\?\') -and $buildFolder -match '^\\\\\?\\[A-Za-z]:\\') { $buildFolder = $buildFolder.Substring(4) }
if ($ToolchainBin) { $env:PATH = $ToolchainBin + [IO.Path]::PathSeparator + $env:PATH }
if ($Nugget.Contains('"') -or $Build -notmatch '^[A-Za-z]+$') { throw 'Invalid build arguments.' }
function Invoke-NativeMake([string]$Arguments) {
$start = New-Object System.Diagnostics.ProcessStartInfo
$start.FileName = (Get-Command $Make -ErrorAction Stop).Source
$start.WorkingDirectory = $buildFolder
$start.UseShellExecute = $false
$start.CreateNoWindow = $true
$start.RedirectStandardOutput = $true
$start.RedirectStandardError = $true
$start.Arguments = '"BUILD=' + $Build + '" "NUGGET_DIR=' + $Nugget.Replace('\','/') + '" ' + $Arguments
$process = [System.Diagnostics.Process]::Start($start)
$stdout = $process.StandardOutput.ReadToEndAsync()
$stderr = $process.StandardError.ReadToEndAsync()
$process.WaitForExit()
Write-Output $stdout.Result
if ($stderr.Result) { [Console]::Error.WriteLine($stderr.Result) }
if ($process.ExitCode -ne 0) { throw "MIPS build failed ($($process.ExitCode))" }
}
Write-Output ('Native build directory: ' + $buildFolder)
$sdkBuildId = [Guid]::NewGuid().ToString('N')
$sdkObjects = '.epok-sdk-objects-' + $sdkBuildId
$sdkArchive = 'sdk/' + $sdkBuildId + '/libpsyqo.a'
$sdkRoot = if ([IO.Path]::IsPathRooted($Nugget)) { $Nugget } else { Join-Path $buildFolder $Nugget }
New-Item -ItemType Directory -Path (Join-Path $sdkRoot ('psyqo/' + $sdkObjects)) | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $buildFolder ('sdk/' + $sdkBuildId)) | Out-Null
$sdkArguments = '"EPOK_CERTIFIED_SDK=' + $sdkArchive + '" "EPOK_SDK_OBJECT_DIRECTORY=' + $sdkObjects + '"'
Invoke-NativeMake ('-B epok-standalone-sdk ' + $sdkArguments)
# Export rebuilds are explicit fresh builds, including preserved-timestamp edits.
Invoke-NativeMake ('-B all ' + $sdkArguments)
