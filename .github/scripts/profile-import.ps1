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
 # Local syntax review uses fixed public fixture data and never imports it.
 $key=[Convert]::ToBase64String([byte[]](1..32))
 $public=[Convert]::ToBase64String([byte[]](33..64))
 $inspection=@"
[Interface]
PrivateKey = $key
Address = 10.55.0.2/32
DNS = 1.1.1.1
[Peer]
PublicKey = $public
AllowedIPs = 0.0.0.0/0, ::/0
Endpoint = 127.0.0.1:51820
"@
 $inputEdit=ImportProofFind $window 'WireGuard configuration to import'
 if (-not $inputEdit) { throw 'Config review input missing' }
 $inputValue=$inputEdit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
 $inputValue.SetValue($inspection)
 Start-Sleep -Milliseconds 300
 (ImportProofFind $window 'Check config').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 300
 if (-not (ImportProofFind $window 'Config syntax checked')) { throw 'Valid config review failed' }
 if (-not (ImportProofFind $window 'Addresses: 1 · DNS servers: 1 · Peers: 1 · Route entries: 2')) { throw 'Config review counts differ' }
 if (-not (ImportProofFind $window 'Default routes: IPv4 and IPv6')) { throw 'Config review missed default route families' }
 # Scroll the real panel so result pixels, rather than UIA alone, can be checked.
 $r=$inputEdit.Current.BoundingRectangle
 [ImportProofNative]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
 for ($i=0;$i -lt 6;$i++) { [ImportProofNative]::mouse_event(0x800,0,0,4294967056,[UIntPtr]::Zero);Start-Sleep -Milliseconds 60 }
 Start-Sleep -Milliseconds 200
 ImportProofCapture $window 'profile-import-check-valid.png'
 $inputEdit=ImportProofFind $window 'WireGuard configuration to import'
 $inputValue=$inputEdit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
 $inputValue.SetValue('invalid-local-fixture')
 Start-Sleep -Milliseconds 300
 if (ImportProofFind $window 'Config syntax checked') { throw 'Editing retained stale config review' }
 (ImportProofFind $window 'Check config').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 if (-not (ImportProofFind $window 'Config could not be checked. Review its sections, keys, addresses and endpoints before importing.')) { throw 'Invalid config review feedback missing' }
 ImportProofCapture $window 'profile-import-check-invalid.png'
 if (@(Get-ChildItem (Join-Path $dir 'profiles') -Filter '*.conf' -ErrorAction SilentlyContinue).Count -ne 0) { throw 'Config review saved a profile' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Config review created a tunnel adapter' }
 $nameEdit=ImportProofFind $window 'Imported profile name'
 $nameValue=$nameEdit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
 $nameValue.SetValue('Keep this name')
 Set-Clipboard -Value 'wax-local-fixture-marker'
 $clipboardBefore=Get-Clipboard -Raw
 (ImportProofFind $window 'Clear pasted config').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 300
 $inputEdit=ImportProofFind $window 'WireGuard configuration to import'
 if (-not $inputEdit.Current.HasKeyboardFocus) { throw 'Clear pasted config did not return editor focus' }
 if ($inputEdit.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value -ne '') { throw 'Clear pasted config retained editor text' }
 if ($nameValue.Current.Value -ne 'Keep this name') { throw 'Clear pasted config changed profile name' }
 if ((Get-Clipboard -Raw) -ne $clipboardBefore) { throw 'Clear pasted config changed system clipboard' }
 if ((ImportProofFind $window 'Clear pasted config').Current.IsEnabled) { throw 'Empty config Clear action enabled' }
 if ((ImportProofFind $window 'Import config').Current.IsEnabled) { throw 'Cleared config Import action enabled' }
 ImportProofCapture $window 'profile-import-cleared.png'
 $nameValue.SetValue('')
 'PASS Clear pasted config empties editor and feedback, returns editor focus, preserves name and system clipboard, disables empty actions without saving' | Out-File verify-results.txt -Append -Encoding utf8
 if ((ImportProofFind $window 'Check config').Current.IsEnabled) { throw 'Empty config review action enabled' }
 if (ImportProofFind $window 'Config could not be checked. Review its sections, keys, addresses and endpoints before importing.') { throw 'Clearing retained stale config review error' }
 [ImportProofNative]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
 for ($i=0;$i -lt 8;$i++) { [ImportProofNative]::mouse_event(0x800,0,0,480,[UIntPtr]::Zero);Start-Sleep -Milliseconds 60 }
 'PASS local config review counts and IPv4/IPv6 defaults, edit invalidation, generic invalid feedback, empty disabled and no saved profile or adapter' | Out-File verify-results.txt -Append -Encoding utf8
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
 $shortcuts=ImportProofFind $window 'Show keyboard shortcuts'
 if (-not $shortcuts) { throw 'Keyboard reference Show action missing' }
 $shortcuts.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 for ($i=0;$i -lt 20;$i++) { Start-Sleep -Milliseconds 150; if (ImportProofFind $window 'Find a profile') { break } }
 ImportProofCapture $window 'profile-import-keyboard-reference.png'
 if (-not (ImportProofFind $window 'Find a profile')) { throw 'Keyboard reference did not open accessibly' }
 $hide=ImportProofFind $window 'Hide keyboard shortcuts'
 if (-not $hide) { throw 'Keyboard reference Hide action missing' }
 $hide.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 if (ImportProofFind $window 'Find a profile') { throw 'Keyboard reference did not hide accessibly' }
 ImportProofCapture $window 'profile-import-keyboard-hidden.png'
 'PASS keyboard reference accessible Show and Hide actions' | Out-File verify-results.txt -Append -Encoding utf8
 'PASS quick picker accessible open and select, persisted selection without connection, About keyboard reference' | Out-File verify-results.txt -Append -Encoding utf8
 # Pinned-only is a session-local list filter, not a profile selection.
 (ImportProofFind $window 'Profiles').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 (ImportProofFind $window 'Pin').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 (ImportProofFind $window 'Pinned only').GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
 Start-Sleep -Milliseconds 250
 if (-not (ImportProofFind $window 'Chosen profile')) { throw 'Pinned filter missed pinned profile' }
 if (ImportProofFind $window 'Chosen profile - 2') { throw 'Pinned filter retained unpinned profile' }
 ImportProofCapture $window 'profile-import-pinned-only.png'
 # Find profiles is also the visible label. Ctrl+F identifies the actual edit.
 [ImportProofNative]::SetForegroundWindow([IntPtr]$window.Current.NativeWindowHandle) | Out-Null
 [System.Windows.Forms.SendKeys]::SendWait('^f')
 Start-Sleep -Milliseconds 250
 $search=[System.Windows.Automation.AutomationElement]::FocusedElement
 if ($search.Current.ControlType -ne [System.Windows.Automation.ControlType]::Edit) { throw 'Pinned filter search edit did not receive focus' }
 $searchValue=$search.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern)
 $searchValue.SetValue('no-match-xyz')
 Start-Sleep -Milliseconds 250
 if (-not (ImportProofFind $window 'No matching profiles')) { throw 'Pinned filter did not combine with search' }
 $searchValue.SetValue('')
 Start-Sleep -Milliseconds 250
 (ImportProofFind $window 'Unpin').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 250
 if (-not (ImportProofFind $window 'No pinned profiles')) { throw 'Unpin did not refresh pinned filter' }
 ImportProofCapture $window 'profile-import-no-pinned.png'
 (ImportProofFind $window 'Pinned only').GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
 Start-Sleep -Milliseconds 250
 if (-not (ImportProofFind $window 'Chosen profile - 2')) { throw 'Disabling pinned filter did not restore rows' }
 $saved=Get-Content (Join-Path $dir 'settings.json') -Raw | ConvertFrom-Json
 if ($saved.last_profile -ne 'Chosen profile') { throw 'Pinned filter changed selection' }
 if ((Get-FileHash $original -Algorithm SHA256).Hash -ne $originalHash) { throw 'Pinned filter changed profile config' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Pinned filter connected unexpectedly' }
 'PASS pinned-only filter combines with search, refreshes after unpin, restores all rows without selection/config/adapter changes' | Out-File verify-results.txt -Append -Encoding utf8
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
 # Type into the actual focused edit. Text-field spaces must never connect.
 $search=ImportProofFind $window 'Find quick profile'
 if (-not $search) { throw 'Quick picker search edit missing' }
 if (-not $search.Current.HasKeyboardFocus) { throw 'Quick picker did not focus search on opening' }
 [System.Windows.Forms.SendKeys]::SendWait(' OFFICE 24 ')
 Start-Sleep -Milliseconds 350
 if (-not (ImportProofFind $window 'Office 24')) { throw 'Quick picker name filter missed final profile' }
 if (ImportProofFind $window 'Office 01') { throw 'Quick picker name filter retained unrelated row' }
 ImportProofCapture $window 'profile-import-picker-search.png'
 [System.Windows.Forms.SendKeys]::SendWait('^f')
 Start-Sleep -Milliseconds 250
 $search=ImportProofFind $window 'Find quick profile'
 if (-not $search -or -not $search.Current.HasKeyboardFocus) { throw 'Ctrl+F left the open quick picker' }
 [System.Windows.Forms.SendKeys]::SendWait('{ESC}')
 Start-Sleep -Milliseconds 250
 if (ImportProofFind $window 'Find quick profile') { throw 'Escape did not dismiss the focused quick picker' }
 $switch=ImportProofFind $window 'Switch profile'
 if (-not $switch.Current.HasKeyboardFocus) { throw 'Escape did not return focus to Switch profile' }
 ImportProofCapture $window 'profile-import-picker-escape.png'
 [System.Windows.Forms.SendKeys]::SendWait('{ENTER}')
 Start-Sleep -Milliseconds 250
 $search=ImportProofFind $window 'Find quick profile'
 if (-not $search -or -not $search.Current.HasKeyboardFocus) { throw 'Enter did not reopen and focus quick search' }
 if ($search.GetCurrentPattern([System.Windows.Automation.ValuePattern]::Pattern).Current.Value -ne '') { throw 'Reopening quick search did not reset query' }
 'PASS quick picker autofocus, contextual Ctrl+F, Escape while typing, returned Switch focus and Enter reopen' | Out-File verify-results.txt -Append -Encoding utf8
 [System.Windows.Forms.SendKeys]::SendWait('^a')
 [System.Windows.Forms.SendKeys]::SendWait('no-match-xyz')
 Start-Sleep -Milliseconds 300
 if (-not (ImportProofFind $window 'No matching profiles')) { throw 'Quick picker empty result missing' }
 ImportProofCapture $window 'profile-import-picker-empty.png'
 [System.Windows.Forms.SendKeys]::SendWait('^a')
 [System.Windows.Forms.SendKeys]::SendWait('127.0.0.1:51820')
 Start-Sleep -Milliseconds 300
 if (-not (ImportProofFind $window 'Chosen profile')) { throw 'Quick picker endpoint search missed matching row' }
 if (ImportProofFind $window 'No matching profiles') { throw 'Quick picker endpoint search kept empty result' }
 (ImportProofFind $window 'Clear').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 300
 if (-not (ImportProofFind $window 'Office 01')) { throw 'Quick picker Clear did not restore rows' }
 $saved=Get-Content (Join-Path $dir 'settings.json') -Raw | ConvertFrom-Json
 if ($saved.last_profile -ne 'Chosen profile') { throw 'Quick picker search changed selected profile' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Quick picker search connected unexpectedly' }
 ImportProofCapture $window 'profile-import-picker-cleared.png'
 'PASS quick picker name/endpoint search, empty result, Clear and unchanged selection without adapter' | Out-File verify-results.txt -Append -Encoding utf8
 $first=ImportProofFind $window 'Chosen profile'
 $r=$first.Current.BoundingRectangle
 [ImportProofNative]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+$r.Height/2)) | Out-Null
 $last=$null
 for ($i=0;$i -lt 24;$i++) {
  $last=ImportProofFind $window 'Office 24'
  if ($last -and -not $last.Current.IsOffscreen) { break }
  [ImportProofNative]::mouse_event(0x800,0,0,4294967056,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 150
 }
 # show_rows keeps a buffered row in UIA before every pixel is visible.
 # Move another wheel step so the final row is fully inside the viewport.
 [ImportProofNative]::mouse_event(0x800,0,0,4294967056,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 250
 $last=ImportProofFind $window 'Office 24'
 ImportProofCapture $window 'profile-import-long-picker-scrolled.png' 
 if (-not $last -or $last.Current.IsOffscreen) { throw 'Long picker could not scroll to final profile' }
 $last.GetCurrentPattern([System.Windows.Automation.TogglePattern]::Pattern).Toggle()
 Start-Sleep -Milliseconds 350
 $saved=Get-Content (Join-Path $dir 'settings.json') -Raw | ConvertFrom-Json
 if ($saved.last_profile -ne 'Office 24') { throw 'Long picker final selection was not saved' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Long picker connected unexpectedly' }
 ImportProofCapture $window 'profile-import-long-selected.png'
 'PASS 26-profile quick picker bounded, Connect on-screen, scrolled to final row and saved selection without adapter' | Out-File verify-results.txt -Append -Encoding utf8
 # Profiles itself draws only the visible fixed-height rows plus a small buffer.
 (ImportProofFind $window 'Profiles').GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 300
 $nodes=$window.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition)
 $officeRows=@($nodes | Where-Object { $_.Current.Name -match '^Office [0-9]{2}$' })
 if ($officeRows.Count -gt 8) { throw 'Profiles rendered the entire long list instead of visible rows' }
 $first=ImportProofFind $window 'Chosen profile'
 if (-not $first) { throw 'Profiles virtual list first row missing' }
 $r=$first.Current.BoundingRectangle
 [ImportProofNative]::SetCursorPos([int]($r.X+$r.Width/2),[int]($r.Y+24)) | Out-Null
 ImportProofCapture $window 'profile-import-list-first.png'
 $last=$null
 for ($i=0;$i -lt 32;$i++) {
  $last=ImportProofFind $window 'Office 24'
  if ($last -and -not $last.Current.IsOffscreen) { break }
  [ImportProofNative]::mouse_event(0x800,0,0,4294967056,[UIntPtr]::Zero)
  Start-Sleep -Milliseconds 100
 }
 [ImportProofNative]::mouse_event(0x800,0,0,4294967056,[UIntPtr]::Zero)
 Start-Sleep -Milliseconds 250
 $last=ImportProofFind $window 'Office 24'
 if (-not $last -or $last.Current.IsOffscreen) { throw 'Profiles could not scroll to last row' }
 $labelY=$last.Current.BoundingRectangle.Y
 $nodes=$window.FindAll([System.Windows.Automation.TreeScope]::Descendants,[System.Windows.Automation.Condition]::TrueCondition)
 $pin=$nodes | Where-Object { $_.Current.Name -eq 'Pin' -and -not $_.Current.IsOffscreen -and [Math]::Abs($_.Current.BoundingRectangle.Y-$labelY) -lt 25 } | Select-Object -First 1
 if (-not $pin) { throw 'Last profile row Pin action missing' }
 ImportProofCapture $window 'profile-import-list-last.png'
 $pin.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern).Invoke()
 Start-Sleep -Milliseconds 300
 $saved=Get-Content (Join-Path $dir 'settings.json') -Raw | ConvertFrom-Json
 if ($saved.pinned_profiles -notcontains 'Office 24' -or $saved.pinned_profiles -contains 'Office 23') { throw 'Virtual row action targeted the wrong profile' }
 if ($saved.last_profile -ne 'Office 24') { throw 'Virtual row Pin changed selected profile' }
 if ((Get-FileHash $original -Algorithm SHA256).Hash -ne $originalHash) { throw 'Virtual list changed profile config' }
 if (Get-NetAdapter -Name 'waX VPN' -ErrorAction SilentlyContinue) { throw 'Virtual list connected unexpectedly' }
 'PASS Profiles bounded visible rows, scroll to final profile, correct Pin target and unchanged selection/config without adapter' | Out-File verify-results.txt -Append -Encoding utf8


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
