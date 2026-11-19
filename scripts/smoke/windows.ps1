# Mathéo's smoke test on Windows (.github/workflows/smoke.yml): started on the runner's desktop, his
# journal read to find him, hovered, thrown, his menu opened, the pointer moved away, then the gallery
# of his looks; a screenshot at each step. Everything in smoke-out/ (screenshots, journal).
param([string]$Exe = "target\debug\matheo.exe")
$ErrorActionPreference = "Continue"

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Input {
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
[Input]::SetProcessDPIAware() | Out-Null

$out = "smoke-out"
New-Item -ItemType Directory -Force $out | Out-Null
$log = Join-Path $env:APPDATA "Matheo-dev\config\matheo.log"

function Shot([string]$name) {
    $b = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($b.Left, $b.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path $out "$name.png"))
    $g.Dispose(); $bmp.Dispose()
    Write-Host "screenshot $name"
}

# Where he stands (his pot's foot), the journal's last word on it.
function Foot {
    $m = Select-String -Path $log -Pattern 'Some\(\((-?[\d.]+), (-?[\d.]+)\)\) ; ses' | Select-Object -Last 1
    if (-not $m) { return $null }
    return @([int][double]$m.Matches[0].Groups[1].Value, [int][double]$m.Matches[0].Groups[2].Value)
}

# The runner's pointer can't be moved (no mouse in its session): Mathéo, a debug build, reads it
# from this file instead (MATHEO_TEST_POINTER, see test_pointer in scripts/sync/overlay_head.rs).
$pointer = Join-Path (Get-Location) "smoke-pointer.txt"
$env:MATHEO_TEST_POINTER = $pointer
$script:at = @(200, 200)
$script:buttons = ""
function Tell { Set-Content -Path $pointer -Value "$($script:at[0]) $($script:at[1]) $($script:buttons)" -NoNewline }
function MoveTo([int]$x, [int]$y) { $script:at = @($x, $y); Tell }
function Down { $script:buttons = "left"; Tell }
function Up { $script:buttons = ""; Tell }
function RightClick { $script:buttons = "right"; Tell; Start-Sleep -Milliseconds 150; $script:buttons = ""; Tell }

$b = [System.Windows.Forms.SystemInformation]::VirtualScreen
Write-Host "screen: $($b.Width) x $($b.Height) at $($b.Left), $($b.Top)"
Tell
Shot "00-desktop"

$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds 10
Shot "01-started"
if ($p.HasExited) { Write-Host "Mathéo exited: $($p.ExitCode)" }

$foot = Foot
Write-Host "foot: $foot"
if ($foot) {
    $x = $foot[0]; $y = $foot[1] - 15
    MoveTo $x $y
    Start-Sleep -Seconds 1
    Shot "02-hover"

    # Grabbed, thrown up and to the left.
    Down
    Start-Sleep -Milliseconds 200
    for ($i = 1; $i -le 12; $i++) {
        MoveTo ($x - $i * 12) ($y - $i * 25)
        Start-Sleep -Milliseconds 25
    }
    Up
    Start-Sleep -Milliseconds 250
    Shot "03-thrown"
    Start-Sleep -Seconds 5
    Shot "04-landed"

    # His menu: a right click on him.
    $foot = Foot
    Write-Host "foot after the throw: $foot"
    if ($foot) {
        MoveTo $foot[0] ($foot[1] - 15)
        Start-Sleep -Milliseconds 500
        RightClick
        Start-Sleep -Seconds 1
        Shot "05-menu"
        # A click elsewhere closes it.
        MoveTo 300 300
        Start-Sleep -Milliseconds 300
        Down
        Start-Sleep -Milliseconds 150
        Up
        Start-Sleep -Seconds 1
    }
}
MoveTo 100 100
Start-Sleep -Seconds 2
Shot "06-pointer-away"

# Flicker: a burst of screenshots, still then with the pointer moving about (a window showing and
# hiding would differ from one to the next).
New-Item -ItemType Directory -Force (Join-Path $out "burst") | Out-Null
for ($i = 0; $i -lt 40; $i++) {
    if ($i -ge 20) { MoveTo (100 + ($i - 20) * 40) (200 + ($i % 3) * 150) }
    Shot ("burst\{0:d2}" -f $i)
    Start-Sleep -Milliseconds 60
}

if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
Start-Sleep -Seconds 1
if (Test-Path $log) { Copy-Item $log (Join-Path $out "matheo.log") }

# The gallery of his looks and outfits.
$env:MATHEO_GALLERY = "1"
$g = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds 8
Shot "07-gallery"
if (-not $g.HasExited) { Stop-Process -Id $g.Id -Force }
Start-Sleep -Seconds 1
if (Test-Path $log) { Copy-Item $log (Join-Path $out "matheo-gallery.log") }
