$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class CpuNative {
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h,uint m,IntPtr w,IntPtr l);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int c);
}
'@
$exe = Join-Path $PWD 'target/release/wax-vpn.exe'
$dir = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'wax-vpn'
$log = Join-Path $dir 'startup.log'
function Find-Window($pidValue) {
 $condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty,$pidValue)
 [System.Windows.Automation.AutomationElement]::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children,$condition)
}
function Find-Name($window,$name) {
 $condition = New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$name)
 $window.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$condition)
}
function Capture($window,$path) {
 $r=$window.Current.BoundingRectangle
 $image=New-Object Drawing.Bitmap([int]$r.Width,[int]$r.Height)
 $g=[Drawing.Graphics]::FromImage($image)
 $g.CopyFromScreen([int]$r.X,[int]$r.Y,0,0,$image.Size)
 $image.Save((Join-Path $PWD $path));$g.Dispose();$image.Dispose()
}
foreach ($mode in @('tray','fallback')) {
 $argsForMode=@('--software-graphics');if ($mode -eq 'fallback') { $argsForMode+='--no-tray' }
 New-Item -ItemType Directory -Force $dir | Out-Null
 Remove-Item $log -ErrorAction SilentlyContinue
 $p=Start-Process $exe -ArgumentList $argsForMode -PassThru
 try {
  $window=$null
  for ($i=0;$i -lt 80;$i++) {
   Start-Sleep -Milliseconds 200
   $p.Refresh();if ($p.HasExited) { throw "CPU $mode exited early: $($p.ExitCode)" }
   $window=Find-Window $p.Id
   if ($window -and (Find-Name $window 'Profiles')) { break }
  }
  if (-not $window -or -not (Find-Name $window 'Profiles')) { throw "CPU $mode accessible rail missing" }
  $hwnd=[IntPtr]$window.Current.NativeWindowHandle
  Capture $window "cpu-$mode-start.png"
  $profiles=Find-Name $window 'Profiles'
  $invoke=$profiles.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
  $invoke.Invoke();Start-Sleep -Milliseconds 500
  if (-not (Find-Name $window 'Import')) { throw "CPU $mode tab input did not change panel" }
  Capture $window "cpu-$mode-input.png"
  Start-Sleep -Seconds 10
  if (-not (Select-String -Path $log -SimpleMatch 'ten UI frames entered' -Quiet)) { throw 'CPU ten-frame marker missing' }
  if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Startup-only CPU gate created an adapter' }
  [CpuNative]::PostMessage($hwnd,0x10,[IntPtr]::Zero,[IntPtr]::Zero) | Out-Null
  Start-Sleep -Seconds 2
  $p.Refresh();if ($p.HasExited) { throw "CPU $mode close bypassed tray/fallback" }
  if ($mode -eq 'fallback') {
   if (-not [CpuNative]::IsIconic($hwnd)) { throw 'CPU fallback did not minimize' }
   [CpuNative]::ShowWindow($hwnd,9) | Out-Null
  } else {
   if ([CpuNative]::IsWindowVisible($hwnd)) { throw 'CPU tray close did not hide window' }
  }
  # Second instance must restore the SAME primary window.
  $second=Start-Process $exe -ArgumentList '--software-graphics' -PassThru
  if (-not $second.WaitForExit(5000)) { throw 'CPU second instance did not exit' }
  Start-Sleep -Seconds 2
  $window=Find-Window $p.Id
  if (-not $window -or [IntPtr]$window.Current.NativeWindowHandle -ne $hwnd -or $window.Current.IsOffscreen -or -not (Find-Name $window 'Profiles')) { throw 'CPU primary HWND restore failed' }
  Capture $window "cpu-$mode-restored.png"
  & $exe --quit
  if (-not $p.WaitForExit(15000)) { throw "CPU $mode graceful quit timed out" }
  "PASS CPU $mode launch, visible accessible rail, UIA tab input, ten frames, no adapter, close handling, same-HWND restore, graceful quit" | Out-File cpu-results.txt -Append -Encoding utf8
 } finally {
  Copy-Item $log "cpu-$mode.log" -ErrorAction SilentlyContinue
  if (-not $p.HasExited) { $p.Kill();$p.WaitForExit() }
 }
}
Get-CimInstance Win32_OperatingSystem | Select-Object Caption,Version | Format-List | Out-File cpu-host.txt
Get-CimInstance Win32_VideoController | Select-Object Name,DriverVersion | Format-List | Out-File cpu-host.txt -Append
