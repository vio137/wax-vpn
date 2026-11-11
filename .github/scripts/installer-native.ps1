# Elevated isolated runner only. Disabled/no-network fixtures are local and temporary.
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true') { throw 'Isolated GitHub runner required' }
$dir = Join-Path $env:ProgramFiles 'waX VPN'
$exe = Join-Path $dir 'waX VPN.exe'
$quitLog = Join-Path $env:APPDATA 'wax-vpn\wax-debug.log'
$setup = (Get-ChildItem installer-download -Filter '*.exe' | Select-Object -First 1).FullName
$fixture = Join-Path $env:RUNNER_TEMP 'wax-native-process-fixture'
$other = Join-Path $fixture 'unrelated'
$hung = Join-Path $fixture 'hung-install'
$helper = Join-Path $fixture 'wax-installer.exe'
New-Item -ItemType Directory -Force $other,$hung | Out-Null
$sentinel = $null; $hungProcess = $null; $gui = $null
$quitEvent = $null; $instanceMutex = $null
function Note($text) { Write-Host $text; $text | Add-Content installer-native-results.txt }
try {
  if (Test-Path $dir) { throw 'Previous smoke test did not uninstall' }
  Invoke-ObservedInstaller $setup 'native-fresh-install'
  Copy-Item (Join-Path $dir 'wax.exe') $helper
  # Ping is used only as a local non-GUI fixture with the same executable name.
  # These processes make loopback ICMP requests, not external network calls.
  Copy-Item "$env:SystemRoot\System32\ping.exe" (Join-Path $other 'waX VPN.exe')
  Copy-Item "$env:SystemRoot\System32\ping.exe" (Join-Path $hung 'waX VPN.exe')
  $sentinel = Start-Process (Join-Path $other 'waX VPN.exe') -ArgumentList '-t','127.0.0.1' -WindowStyle Hidden -PassThru
  if (Test-Path $quitLog) { Clear-Content $quitLog }
  $gui = Start-Process $exe -PassThru
  Start-Sleep -Seconds 3
  if ($gui.HasExited) { throw 'Installed GUI exited before running upgrade' }
  Invoke-ObservedInstaller $setup 'native-running-gui-upgrade'
  if (-not $gui.WaitForExit(10000)) { throw 'Upgrade did not stop installed GUI' }
  $sentinel.Refresh(); if ($sentinel.HasExited) { throw 'Upgrade killed unrelated same-name process' }
  if (-not (Test-Path $quitLog) -or (Get-Content $quitLog -Raw) -notmatch 'quit: signal consumed') { throw 'Upgrade did not use graceful PID-scoped quit' }
  Copy-Item $quitLog 'installer-native-upgrade-quit.txt'
  Note 'PASS running-GUI upgrade used graceful PID-scoped quit and preserved unrelated same-name process'

  if (Test-Path $quitLog) { Clear-Content $quitLog }
  $gui = Start-Process $exe -PassThru
  Start-Sleep -Seconds 3
  if ($gui.HasExited) { throw 'Installed GUI exited before uninstall' }
  Invoke-ObservedInstaller (Join-Path $dir 'uninstall.exe') 'native-running-gui-uninstall'
  if (-not $gui.WaitForExit(10000)) { throw 'Uninstall did not stop installed GUI' }
  $sentinel.Refresh(); if ($sentinel.HasExited) { throw 'Uninstall killed unrelated same-name process' }
  if (Test-Path $exe) { throw 'Uninstall retained installed GUI' }
  if (-not (Test-Path $quitLog) -or (Get-Content $quitLog -Raw) -notmatch 'quit: signal consumed') { throw 'Uninstall did not use graceful PID-scoped quit' }
  Copy-Item $quitLog 'installer-native-uninstall-quit.txt'
  Note 'PASS running-GUI uninstall used graceful PID-scoped quit and preserved unrelated same-name process'

  $hungProcess = Start-Process (Join-Path $hung 'waX VPN.exe') -ArgumentList '-t','127.0.0.1' -WindowStyle Hidden -PassThru
  Start-Sleep -Milliseconds 500
  $quitEvent = [Threading.EventWaitHandle]::new($false, [Threading.EventResetMode]::AutoReset, 'Local\waXVPN.Quit')
  $instanceMutex = [Threading.Mutex]::new($false, 'Local\waXVPN.Instance')
  $timer = [Diagnostics.Stopwatch]::StartNew()
  & $helper installer-stop $hung
  if ($LASTEXITCODE -ne 0) { throw 'Native forced cleanup helper failed' }
  if (-not $hungProcess.WaitForExit(10000)) { throw 'Native cleanup did not stop verified hung fixture' }
  if ($timer.Elapsed.TotalSeconds -gt 25) { throw 'Native fallback exceeded bounded wait' }
  $sentinel.Refresh(); if ($sentinel.HasExited) { throw 'Fallback killed unrelated same-name process' }
  if ($quitEvent.WaitOne(0)) { throw 'Global quit event signaled while another copy exists' }
  Note 'PASS native bounded fallback stopped exact-path nonresponsive fixture, preserved unrelated same-name process and did not signal global quit'
} finally {
  if ($quitEvent) { $quitEvent.Dispose() }; if ($instanceMutex) { $instanceMutex.Dispose() }
  foreach ($p in @($gui,$hungProcess,$sentinel)) { if ($p) { $p.Refresh(); if (-not $p.HasExited) { $p.Kill(); $p.WaitForExit() } } }
  if (Test-Path (Join-Path $dir 'uninstall.exe')) { $p=Start-Process (Join-Path $dir 'uninstall.exe') -ArgumentList '/S' -PassThru; $null=$p.WaitForExit(30000) }
  Remove-Item $fixture -Recurse -Force -ErrorAction SilentlyContinue
}
