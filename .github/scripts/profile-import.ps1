# CI-only local-file import proof. No tunnel or external service is used.
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'This fixture may run only on a disposable CI runner' }
Add-Type -AssemblyName UIAutomationClient
Add-Type -AssemblyName UIAutomationTypes
Add-Type -AssemblyName System.Windows.Forms
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class ImportProofNative {
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
}
'@
function ImportProofFind($root,$name) {
 $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,$name)))
}
function ImportProofCapture($window,$path) {
 $r=$window.Current.BoundingRectangle
 $b=New-Object Drawing.Bitmap([int]$r.Width,[int]$r.Height);$g=[Drawing.Graphics]::FromImage($b)
 try { $g.CopyFromScreen([int]$r.X,[int]$r.Y,0,0,$b.Size);$b.Save((Join-Path $PWD $path)) } finally { $g.Dispose();$b.Dispose() }
}
function ImportProofDialog($pidValue) {
 $root=[System.Windows.Automation.AutomationElement]::RootElement
 $c=New-Object System.Windows.Automation.AndCondition((New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty,$pidValue)),(New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::NameProperty,'Import a WireGuard profile')))
 $root.FindFirst([System.Windows.Automation.TreeScope]::Descendants,$c)
}
function ImportProofOpen($window,$pidValue) {
 [ImportProofNative]::SetForegroundWindow([IntPtr]$window.Current.NativeWindowHandle) | Out-Null
 # Cancel closes a native modal loop before egui has redrawn its panel. Let
 # layout settle, then reacquire the element rather than clicking old bounds.
 Start-Sleep -Milliseconds 750
 $window=[System.Windows.Automation.AutomationElement]::FromHandle([IntPtr]$window.Current.NativeWindowHandle)
 $button=ImportProofFind $window 'Choose .conf file'
 if (-not $button) { throw 'Choose .conf file control missing' }
 $r=$button.Current.BoundingRectangle
 Write-Host "Picker button bounds=$r offscreen=$($button.Current.IsOffscreen) enabled=$($button.Current.IsEnabled) window=$($window.Current.BoundingRectangle)"
 ImportProofCapture $window 'profile-import-before-picker.png'
 # The chooser now starts asynchronously, so Invoke cannot block this test
 # behind its modal loop. Use the actual accessible action, not cursor state
 # left behind by native modal cancellation.
 $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 for ($i=0;$i -lt 40;$i++) { Start-Sleep -Milliseconds 125; $d=ImportProofDialog $pidValue;if ($d) { return $d } }
 ImportProofCapture $window 'profile-import-open-failed.png'
 $nodes=$window.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition)
 $nodes | ForEach-Object { "$($_.Current.ControlType.ProgrammaticName) $($_.Current.Name) offscreen=$($_.Current.IsOffscreen) bounds=$($_.Current.BoundingRectangle)" } | Out-File profile-import-uia.txt
 Get-Content profile-import-uia.txt | Write-Host
 throw 'Native file picker did not appear'
}
$exe=Join-Path $PWD 'bin/wax-vpn.exe'
$dir=Join-Path $env:APPDATA 'wax-vpn'
$backup="$dir-import-proof-$([Guid]::NewGuid().ToString('N'))"
$fixture=Join-Path $env:RUNNER_TEMP 'Chosen profile.conf'
$hadDirectory=Test-Path $dir
if ($hadDirectory) { Move-Item $dir $backup }
$p=$null
try {
 New-Item -ItemType Directory -Force $dir | Out-Null
 Set-Content (Join-Path $dir 'settings.json') '{"auto_reconnect":false,"reduced_motion":true}'
 $p=Start-Process $exe -ArgumentList '--no-tray' -PassThru
 $condition=New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ProcessIdProperty,$p.Id)
 $window=$null
 for ($i=0;$i -lt 60;$i++) {
  Start-Sleep -Milliseconds 150
  $window=[System.Windows.Automation.AutomationElement]::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children,$condition)
  if ($window -and (ImportProofFind $window 'Import profile')) { break }
 }
 $button=if ($window) { ImportProofFind $window 'Import profile' }
 if (-not $button) { throw 'First-run Import profile action missing' }
 ImportProofCapture $window 'profile-import-home.png'
 $button.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 300
 if (-not (ImportProofFind $window 'Choose .conf file')) { throw 'First-run action did not open Profiles' }
 $dialog=ImportProofOpen $window $p.Id
 ImportProofCapture $dialog 'profile-import-picker.png'
 $cancel=ImportProofFind $dialog 'Cancel'
 if (-not $cancel) { throw 'Native picker Cancel control missing' }
 $cancel.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 if (@(Get-ChildItem (Join-Path $dir 'profiles') -Filter '*.conf' -ErrorAction SilentlyContinue).Count -ne 0) { throw 'Cancelling imported a profile' }
 $key=[Convert]::ToBase64String([byte[]](1..32))
 $public=[Convert]::ToBase64String([byte[]](33..64))
 @"
[Interface]
PrivateKey = $key
Address = 10.55.0.2/32
[Peer]
PublicKey = $public
AllowedIPs = 10.55.0.1/32
Endpoint = 127.0.0.1:51820
"@ | Set-Content $fixture
 $fixtureHash=(Get-FileHash $fixture -Algorithm SHA256).Hash
 $originalHash=$null
 foreach ($attempt in 1..2) {
  $dialog=ImportProofOpen $window $p.Id
  [ImportProofNative]::SetForegroundWindow([IntPtr]$dialog.Current.NativeWindowHandle) | Out-Null
  [System.Windows.Forms.SendKeys]::SendWait('%n')
  Start-Sleep -Milliseconds 150
  $edit=[System.Windows.Automation.AutomationElement]::FocusedElement
  $value=$edit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
  $value.SetValue($fixture)
  [System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
  for ($i=0;$i -lt 40;$i++) { Start-Sleep -Milliseconds 150; $files=@(Get-ChildItem (Join-Path $dir 'profiles') -Filter '*.conf' -ErrorAction SilentlyContinue);if ($files.Count -eq $attempt) { break } }
  if ($files.Count -ne $attempt) { throw "Native picker import count mismatch on attempt $attempt" }
  if ($attempt -eq 1) { $original=$files[0].FullName;$originalHash=(Get-FileHash $original -Algorithm SHA256).Hash }
  if ((Get-FileHash $original -Algorithm SHA256).Hash -ne $originalHash) { throw 'Duplicate import overwrote the original profile' }
 }
 if ((Get-FileHash $fixture -Algorithm SHA256).Hash -ne $fixtureHash) { throw 'Import modified the selected source file' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Local import proof created a tunnel adapter' }
 ImportProofCapture $window 'profile-import-result.png'
 'PASS first-run Import action, native picker cancel, chosen-file import, duplicate preservation, unchanged source and no adapter' | Out-File verify-results.txt -Append -Encoding utf8
 $connect=ImportProofFind $window 'Connect'
 $connect.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 350
 $switch=ImportProofFind $window 'Switch profile'
 if (-not $switch) { throw 'Accessible quick profile switch action missing' }
 $switch.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 350
 ImportProofCapture $window 'profile-import-quick-picker.png'
 $choice=ImportProofFind $window 'Chosen profile'
 if (-not $choice) { throw 'Quick picker original profile row missing' }
 # egui selected buttons expose UIA TogglePattern to report their state.
 $choice.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
 Start-Sleep -Milliseconds 350
 $saved=Get-Content (Join-Path $dir 'settings.json') -Raw | ConvertFrom-Json
 if ($saved.last_profile -ne 'Chosen profile') { throw 'Quick picker selection was not saved' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Quick picker selection connected unexpectedly' }
 ImportProofCapture $window 'profile-import-quick-selected.png'
 $about=ImportProofFind $window 'About'
 $about.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 350
 $shortcuts=ImportProofFind $window 'Keyboard shortcuts'
 if (-not $shortcuts) { throw 'Keyboard reference missing' }
 # egui exposes expanded state but its response listens to Click, not an
 # Expand action. Exercise the visible disclosure with a normal mouse click.
 $r=$shortcuts.Current.BoundingRectangle
 [ImportProofNative]::SetForegroundWindow([IntPtr]$window.Current.NativeWindowHandle) | Out-Null
 [ImportProofNative]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
 [ImportProofNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 100
 [ImportProofNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
 for ($i=0;$i -lt 20;$i++) { Start-Sleep -Milliseconds 150; if (ImportProofFind $window 'Find a profile') { break } }
 ImportProofCapture $window 'profile-import-keyboard-reference.png'
 if (-not (ImportProofFind $window 'Find a profile')) { throw 'Keyboard reference did not expand' }
 ImportProofCapture $window 'profile-import-keyboard-reference.png'
 'PASS quick picker accessible open and select, persisted selection without connection, About keyboard reference' | Out-File verify-results.txt -Append -Encoding utf8
 # A long list must stay bounded and let a late profile be chosen locally.
 foreach ($n in 1..24) { Copy-Item $fixture (Join-Path (Join-Path $dir 'profiles') ('Office {0:D2}.conf' -f $n)) }
 (ImportProofFind $window 'Profiles').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 (ImportProofFind $window 'Reload profiles').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 (ImportProofFind $window 'Connect').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 (ImportProofFind $window 'Switch profile').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 $first=ImportProofFind $window 'Chosen profile'
 if (-not $first) { throw 'Long quick picker first row missing' }
 $r=$first.Current.BoundingRectangle
 $button=ImportProofFind $window 'Connect to Chosen profile'
 if (-not $button -or $button.Current.IsOffscreen) { throw 'Long picker pushed Connect off-screen' }
 ImportProofCapture $window 'profile-import-long-picker.png'
 [ImportProofNative]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
 $last=$null
 for ($i=0;$i -lt 24;$i++) {
  $last=ImportProofFind $window 'Office 24'
  if ($last -and -not $last.Current.IsOffscreen) { break }
  [ImportProofNative]::mouse_event(0x800,0,0,4294967056,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150
 }
 ImportProofCapture $window 'profile-import-long-picker-scrolled.png'
 if (-not $last -or $last.Current.IsOffscreen) { throw 'Long picker could not scroll to final profile' }
 $last.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
 Start-Sleep -Milliseconds 350
 $saved=Get-Content (Join-Path $dir 'settings.json') -Raw | ConvertFrom-Json
 if ($saved.last_profile -ne 'Office 24') { throw 'Long picker final selection was not saved' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Long picker connected unexpectedly' }
 ImportProofCapture $window 'profile-import-long-selected.png'
 'PASS 26-profile quick picker bounded, Connect on-screen, scrolled to final row and saved selection without adapter' | Out-File verify-results.txt -Append -Encoding utf8


} finally {
 if ($p) {
  $dialog=ImportProofDialog $p.Id
  if ($dialog) { [ImportProofNative]::SetForegroundWindow([IntPtr]$dialog.Current.NativeWindowHandle) | Out-Null;[System.Windows.Forms.SendKeys]::SendWait('{ESC}');Start-Sleep -Milliseconds 200 }
  & $exe --quit
  if (-not $p.WaitForExit(10000)) { $p.Kill();$p.WaitForExit(5000) | Out-Null }
 }
 Remove-Item $fixture -ErrorAction SilentlyContinue
 if (Test-Path $dir) { Remove-Item $dir -Recurse -Force }
 if ($hadDirectory -and (Test-Path $backup)) { Move-Item $backup $dir }
}
