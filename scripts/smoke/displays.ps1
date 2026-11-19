# The Windows runner given more screens, at other scales, as on our laptops (one screen at 125 %, two
# more at 100 %): Amyuni's virtual display driver (usbmmidd, for tests only, never shipped), each
# screen's scale set through Windows' display configuration (as Settings > Display does).
# Usage: displays.ps1 -Add 2 -Scales 125,100,100
param([int]$Add = 2, [int[]]$Scales = @(125, 100, 100))
$ErrorActionPreference = "Continue"

Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;

public static class Displays {
    [StructLayout(LayoutKind.Sequential)] public struct LUID { public uint Low; public int High; }

    [StructLayout(LayoutKind.Sequential)]
    public struct PATH {
        public LUID sourceAdapter; public uint sourceId; public uint sourceMode; public uint sourceFlags;
        public LUID targetAdapter; public uint targetId; public uint targetMode; public uint tech; public uint rotation;
        public uint scaling; public uint rateNum; public uint rateDen; public uint scanline; public int available; public uint targetFlags;
        public uint flags;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct HEADER { public int type; public int size; public LUID adapter; public uint id; }

    [StructLayout(LayoutKind.Sequential)]
    public struct DPI_GET { public HEADER header; public int min; public int cur; public int max; }

    [StructLayout(LayoutKind.Sequential)]
    public struct DPI_SET { public HEADER header; public int scale; }

    [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
    [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint paths, [Out] PATH[] p, ref uint modes, IntPtr m, IntPtr topology);
    [DllImport("user32.dll")] static extern int DisplayConfigGetDeviceInfo(ref DPI_GET info);
    [DllImport("user32.dll")] static extern int DisplayConfigSetDeviceInfo(ref DPI_SET info);
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();

    // The scales Windows offers, in this order (the « recommended » one is Windows' 0).
    static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

    public static List<PATH> Paths() {
        uint np, nm;
        GetDisplayConfigBufferSizes(2, out np, out nm); // QDC_ONLY_ACTIVE_PATHS
        var paths = new PATH[np];
        IntPtr modes = Marshal.AllocHGlobal((int)nm * 64);
        int r = QueryDisplayConfig(2, ref np, paths, ref nm, modes, IntPtr.Zero);
        Marshal.FreeHGlobal(modes);
        var list = new List<PATH>();
        if (r == 0) for (int i = 0; i < np; i++) list.Add(paths[i]);
        return list;
    }

    // Path `p`'s source at `percent`: the result, and what Windows says before.
    public static string SetScale(PATH p, int percent) {
        var get = new DPI_GET();
        get.header.type = -3; get.header.size = Marshal.SizeOf(typeof(DPI_GET)); get.header.adapter = p.sourceAdapter; get.header.id = p.sourceId;
        int g = DisplayConfigGetDeviceInfo(ref get);
        int recommended = Math.Abs(get.min);
        int want = Array.IndexOf(Steps, percent);
        var set = new DPI_SET();
        set.header.type = -4; set.header.size = Marshal.SizeOf(typeof(DPI_SET)); set.header.adapter = p.sourceAdapter; set.header.id = p.sourceId;
        set.scale = want - recommended;
        int s = DisplayConfigSetDeviceInfo(ref set);
        return "source " + p.sourceId + ": get " + g + " (min " + get.min + ", cur " + get.cur + ", max " + get.max + "), set " + percent + "% -> " + s;
    }
}
"@
[Displays]::SetProcessDPIAware() | Out-Null

function Show([string]$when) {
    Write-Host "--- screens $when"
    foreach ($s in [System.Windows.Forms.Screen]::AllScreens) {
        Write-Host "$($s.DeviceName) $($s.Bounds) primary=$($s.Primary)"
    }
}

Show "before"
$zip = Join-Path $env:RUNNER_TEMP "usbmmidd.zip"
Invoke-WebRequest -Uri "https://www.amyuni.com/downloads/usbmmidd_v2.zip" -OutFile $zip
Write-Host "usbmmidd_v2.zip sha256: $((Get-FileHash $zip).Hash)"
$dir = Join-Path $env:RUNNER_TEMP "usbmmidd"
Expand-Archive $zip -DestinationPath $dir -Force
$installer = Get-ChildItem $dir -Recurse -Filter deviceinstaller64.exe | Select-Object -First 1
Push-Location $installer.DirectoryName
& .\deviceinstaller64.exe install usbmmidd.inf usbmmidd
for ($i = 0; $i -lt $Add; $i++) {
    & .\deviceinstaller64.exe enableidd 1
    Start-Sleep -Seconds 3
}
Pop-Location
Start-Sleep -Seconds 3
Show "with the virtual ones"

$paths = [Displays]::Paths()
Write-Host "$($paths.Count) display paths"
for ($i = 0; $i -lt $paths.Count -and $i -lt $Scales.Count; $i++) {
    Write-Host ([Displays]::SetScale($paths[$i], $Scales[$i]))
}
Start-Sleep -Seconds 3
Show "scaled"
