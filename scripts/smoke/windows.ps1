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
    [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    [DllImport("user32.dll")] public static extern short GetAsyncKeyState(int key);
    [DllImport("user32.dll")] public static extern bool GetCursorPos(out POINT p);
    public struct POINT { public int X; public int Y; }
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

# The pointer moved as a mouse would (SetCursorPos doesn't move it on the runner): absolute, in
# 0..65535 over the primary screen.
function Move([int]$x, [int]$y) {
    $s = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $nx = [uint32][math]::Round($x * 65535 / ($s.Width - 1))
    $ny = [uint32][math]::Round($y * 65535 / ($s.Height - 1))
    [Input]::mouse_event(0x8001, $nx, $ny, 0, [UIntPtr]::Zero)
}

# What the system says of the pointer (to know whether the moves and clicks reach it).
function Pointer([string]$when) {
    $p = New-Object Input+POINT
    [Input]::GetCursorPos([ref]$p) | Out-Null
    $left = ([Input]::GetAsyncKeyState(1) -band 0x8000) -ne 0
    Write-Host "$($when): pointer at $($p.X), $($p.Y), left button down: $left"
}

$b = [System.Windows.Forms.SystemInformation]::VirtualScreen
Write-Host "screen: $($b.Width) x $($b.Height) at $($b.Left), $($b.Top)"
Move 200 200
Shot "00-desktop"

$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds 10
Shot "01-started"
if ($p.HasExited) { Write-Host "Mathéo exited: $($p.ExitCode)" }

$foot = Foot
Write-Host "foot: $foot"
if ($foot) {
    $x = $foot[0]; $y = $foot[1] - 15
    Move $x $y
    Start-Sleep -Seconds 1
    Pointer "hover"
    Shot "02-hover"

    # Grabbed, thrown up and to the left.
    [Input]::mouse_event(0x2, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 200
    Pointer "grabbed"
    for ($i = 1; $i -le 12; $i++) {
        Move ($x - $i * 12) ($y - $i * 25)
        Start-Sleep -Milliseconds 25
    }
    [Input]::mouse_event(0x4, 0, 0, 0, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 250
    Shot "03-thrown"
    Start-Sleep -Seconds 5
    Shot "04-landed"

    # His menu: a right click on him.
    $foot = Foot
    Write-Host "foot after the throw: $foot"
    if ($foot) {
        Move $foot[0] ($foot[1] - 15)
        Start-Sleep -Milliseconds 500
        [Input]::mouse_event(0x8, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Milliseconds 100
        [Input]::mouse_event(0x10, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Seconds 1
        Shot "05-menu"
        # A click elsewhere closes it.
        Move 300 300
        Start-Sleep -Milliseconds 300
        [Input]::mouse_event(0x2, 0, 0, 0, [UIntPtr]::Zero)
        [Input]::mouse_event(0x4, 0, 0, 0, [UIntPtr]::Zero)
        Start-Sleep -Seconds 1
    }
}
Move 100 100
Start-Sleep -Seconds 2
Shot "06-pointer-away"

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
