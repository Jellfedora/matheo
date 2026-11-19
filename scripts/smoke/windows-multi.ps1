# Mathéo over several screens at different scales (after displays.ps1): the pointer going from screen
# to screen, Mathéo carried and dropped on each, his menu opened there; screenshots of all the screens
# at each step, and in bursts while the pointer moves (a window showing and hiding differs from one
# shot to the next). Everything in smoke-out/ (screenshots, journal).
param([string]$Exe = "target\debug\matheo.exe")
$ErrorActionPreference = "Continue"

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System.Runtime.InteropServices;
public static class Dpi { [DllImport("user32.dll")] public static extern bool SetProcessDPIAware(); }
"@
[Dpi]::SetProcessDPIAware() | Out-Null

$out = "smoke-out"
New-Item -ItemType Directory -Force $out | Out-Null
New-Item -ItemType Directory -Force (Join-Path $out "burst") | Out-Null
$log = Join-Path $env:APPDATA "Matheo-dev\config\matheo.log"

function Shot([string]$name) {
    $b = [System.Windows.Forms.SystemInformation]::VirtualScreen
    $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($b.Left, $b.Top, 0, 0, $bmp.Size)
    $bmp.Save((Join-Path $out "$name.png"))
    $g.Dispose(); $bmp.Dispose()
}

# The pointer, in Mathéo's points, through the file he reads (MATHEO_TEST_POINTER).
$pointer = Join-Path (Get-Location) "smoke-pointer.txt"
$env:MATHEO_TEST_POINTER = $pointer
$script:at = @(10, 10)
$script:buttons = ""
function Tell { Set-Content -Path $pointer -Value "$($script:at[0]) $($script:at[1]) $($script:buttons)" -NoNewline }
function MoveTo([double]$x, [double]$y) { $script:at = @([int]$x, [int]$y); Tell }
function Down { $script:buttons = "left"; Tell }
function Up { $script:buttons = ""; Tell }

# Where he stands (his pot's foot), the journal's last word on it.
function Foot {
    $m = Select-String -Path $log -Pattern 'Some\(\((-?[\d.]+), (-?[\d.]+)\)\) ; ses' | Select-Object -Last 1
    if (-not $m) { return $null }
    return @([double]$m.Matches[0].Groups[1].Value, [double]$m.Matches[0].Groups[2].Value)
}

# The screens as Mathéo counts them (points), from his journal's first word on them.
function Screens {
    $line = (Select-String -Path $log -Pattern '^\S+ écrans \[' | Select-Object -First 1).Line
    $found = [regex]::Matches($line, '\(\[\[(-?[\d.]+) (-?[\d.]+)\] - \[(-?[\d.]+) (-?[\d.]+)\]\], ')
    return @($found | ForEach-Object { , @([double]$_.Groups[1].Value, [double]$_.Groups[2].Value, [double]$_.Groups[3].Value, [double]$_.Groups[4].Value) })
}

# A burst of shots while the pointer goes from (x0, y0) to (x1, y1).
$script:bursts = 0
function Sweep([double]$x0, [double]$y0, [double]$x1, [double]$y1) {
    for ($i = 0; $i -le 12; $i++) {
        MoveTo ($x0 + ($x1 - $x0) * $i / 12) ($y0 + ($y1 - $y0) * $i / 12)
        Shot ("burst\{0:d3}" -f $script:bursts)
        $script:bursts++
        Start-Sleep -Milliseconds 40
    }
}

Tell
Shot "00-desktop"
$p = Start-Process -FilePath $Exe -PassThru
Start-Sleep -Seconds 10
Shot "01-started"
$screens = Screens
Write-Host "screens (points): $(($screens | ForEach-Object { '[' + ($_ -join ', ') + ']' }) -join ' ')"

# The pointer from screen to screen, through their middles, there and back.
$centers = @($screens | ForEach-Object { , @((($_[0] + $_[2]) / 2), (($_[1] + $_[3]) / 2)) })
for ($k = 0; $k -lt $centers.Count; $k++) {
    $a = $centers[$k]; $b = $centers[($k + 1) % $centers.Count]
    Sweep $a[0] $a[1] $b[0] $b[1]
}
Shot "02-after-sweeps"

# Carried to each screen and dropped in its middle; a look at where he lands.
for ($k = 0; $k -lt $screens.Count; $k++) {
    $foot = Foot
    if (-not $foot) { break }
    MoveTo $foot[0] ($foot[1] - 15)
    Start-Sleep -Milliseconds 400
    Down
    Start-Sleep -Milliseconds 200
    $c = $centers[$k]
    for ($i = 1; $i -le 10; $i++) {
        MoveTo ($foot[0] + ($c[0] - $foot[0]) * $i / 10) (($foot[1] - 15) + ($c[1] - $foot[1] + 15) * $i / 10)
        Start-Sleep -Milliseconds 30
    }
    Start-Sleep -Milliseconds 300
    Up
    Start-Sleep -Seconds 4
    Shot ("03-dropped-on-{0}" -f $k)
    Write-Host "dropped on screen $k, foot now: $(Foot)"
    # The pointer away to another screen and back: he stays.
    $o = $centers[($k + 1) % $centers.Count]
    Sweep $c[0] $c[1] $o[0] $o[1]
    Shot ("04-pointer-away-from-{0}" -f $k)
}

# His menu, on the screen he ended on.
$foot = Foot
if ($foot) {
    MoveTo $foot[0] ($foot[1] - 15)
    Start-Sleep -Milliseconds 400
    $script:buttons = "right"; Tell; Start-Sleep -Milliseconds 150; $script:buttons = ""; Tell
    Start-Sleep -Seconds 1
    Shot "05-menu"
}

if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
Start-Sleep -Seconds 1
if (Test-Path $log) { Copy-Item $log (Join-Path $out "matheo.log") }
