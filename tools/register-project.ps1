# Optional per-user distribution registration. Portable operation never calls this.
[CmdletBinding(SupportsShouldProcess)]
param(
    [Parameter(Mandatory=$true)][string]$EditorPath,
    [switch]$Unregister,
    [Parameter(DontShow=$true)][string]$TestRegistryRoot = ''
)
$ErrorActionPreference='Stop'
if ($env:OS -ne 'Windows_NT') {throw 'Project file registration supports Windows only.'}
$editor=[IO.Path]::GetFullPath($EditorPath)
if ($editor.Contains('"')) {throw 'Invalid editor path.'}
if (-not $Unregister -and -not (Test-Path -LiteralPath $editor -PathType Leaf)) {throw 'Build or install epok-editor.exe before registering it.'}
$classes='HKCU:\Software\Classes'
if ($TestRegistryRoot) {
    $testPrefix='HKCU:\Software\Epok\AssociationTests\'
    if (-not $TestRegistryRoot.StartsWith($testPrefix,[StringComparison]::OrdinalIgnoreCase)) {throw 'Invalid isolated registration test root.'}
    $testId=[Guid]::Empty
    if (-not [Guid]::TryParse($TestRegistryRoot.Substring($testPrefix.Length),[ref]$testId)) {throw 'Registration test root needs a UUID.'}
    $classes=$TestRegistryRoot
}
$program='Epok.Project'
$programKey=Join-Path $classes $program
$extensionKey=Join-Path $classes '.epokproject'
$openCommand='"'+$editor+'" "%1"'
function Remove-RegistrationValue([string]$path,[string]$name) {
    $writable=[Microsoft.Win32.Registry]::CurrentUser.OpenSubKey($path.Substring(6),$true)
    if ($writable) {try {$writable.DeleteValue($name,$false)} finally {$writable.Dispose()}}
}
if ($Unregister) {
    if (Test-Path -LiteralPath $programKey) {
        $registered=(Get-Item -LiteralPath (Join-Path $programKey 'shell\open\command')).GetValue('')
        if ($registered -ne $openCommand) {throw 'Another Epok installation owns this registration; no values were removed.'}
        if ($PSCmdlet.ShouldProcess($programKey,'Remove this installation file registration')) {
            if (Test-Path -LiteralPath $extensionKey) {
                $key=Get-Item -LiteralPath $extensionKey
                if ($key.GetValue('') -eq $program) {Remove-RegistrationValue $extensionKey ''}
                $with=Join-Path $extensionKey 'OpenWithProgids'
                if (Test-Path -LiteralPath $with) {Remove-RegistrationValue $with $program}
            }
            # Exact, fixed HKCU registry key, never a filesystem directory.
            Remove-Item -LiteralPath $programKey -Recurse
        }
    }
} elseif ($PSCmdlet.ShouldProcess($editor,'Register .epokproject open command and application icon for this user')) {
    foreach ($suffix in @('','DefaultIcon','shell\open\command')) {$path=Join-Path $programKey $suffix; if (-not (Test-Path -LiteralPath $path)) {New-Item -Path $path -Force | Out-Null}}
    Set-Item -LiteralPath $programKey -Value 'Epok Project'
    Set-Item -LiteralPath (Join-Path $programKey 'DefaultIcon') -Value ('"'+$editor+'",0')
    Set-Item -LiteralPath (Join-Path $programKey 'shell\open\command') -Value $openCommand
    if (-not (Test-Path -LiteralPath $extensionKey)) {New-Item -Path $extensionKey -Force | Out-Null}
    $key=Get-Item -LiteralPath $extensionKey
    # Respect an existing user-selected default. Windows Settings can change it.
    if (-not $key.GetValue('')) {Set-Item -LiteralPath $extensionKey -Value $program}
    $with=Join-Path $extensionKey 'OpenWithProgids'
    if (-not (Test-Path -LiteralPath $with)) {New-Item -Path $with -Force | Out-Null}
    New-ItemProperty -LiteralPath $with -Name $program -Value '' -PropertyType String -Force | Out-Null
}
if (-not $WhatIfPreference -and -not $TestRegistryRoot) {
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public static class EpokFileAssociation { [DllImport("shell32.dll")] public static extern void SHChangeNotify(uint eventId,uint flags,IntPtr item1,IntPtr item2); }'
    [EpokFileAssociation]::SHChangeNotify(0x08000000,0,[IntPtr]::Zero,[IntPtr]::Zero)
}
