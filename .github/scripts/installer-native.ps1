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

# Real Windows Defender verdict, evidence only: records the engine's answer and
# never changes it, fails no assertion, and quarantines nothing by itself.
# Scans the installer built by this run and the latest published release
# installer (the file users actually download). RTP blocking the download is
# itself recorded as a flag.
try {
  $mp = Join-Path ${env:ProgramFiles} 'Windows Defender\MpCmdRun.exe'
  if (-not (Test-Path $mp)) {
    Note 'SKIP Defender scan: MpCmdRun.exe not present on this runner'
  } else {
    try { & $mp -SignatureUpdate | Out-Null } catch { Note "INFO Defender signature update failed: $($_.Exception.Message)" }
    try {
      $st = Get-MpComputerStatus
      Note "INFO Defender engine $($st.AMEngineVersion), signatures $($st.AntivirusSignatureVersion) from $($st.AntivirusSignatureLastUpdated), RTP=$($st.RealTimeProtectionEnabled), mode=$($st.AMRunningMode), AV=$($st.AntivirusEnabled)"
    } catch { Note 'INFO Defender version query failed' }
    $scanTargets = @(@{ label = "this run's installer"; path = $setup })
    $publishedReleases = @(
      @{ tag = 'v1.0.30'; file = 'waX-VPN-Setup-v1.0.30-x64.exe'; sha = '16d14753f225d27b7659add9a00614679e1ccc4e6fcac84e6d723a3ab92a1ed3' },
      @{ tag = 'v1.0.29'; file = 'waX-VPN-Setup-v1.0.29-x64.exe'; sha = 'cf198d7dddffe626ff91ad7bc58a77d8494e857c0cd13fa201121d74fcaf0688' }
    )
    Note "INFO Defender scan: fetching $($publishedReleases.Count) published release installers for A/B scan"
    foreach ($rel in $publishedReleases) {
      $url = "https://github.com/vio137/wax-vpn/releases/download/$($rel.tag)/$($rel.file)"
      $published = Join-Path $env:RUNNER_TEMP $rel.file
      $downloadBlocked = $false
      try {
        Invoke-WebRequest -Uri $url -OutFile $published
      } catch {
        if ($_.Exception.Message -match 'virus') { Note "FAIL Defender real-time protection removed the published $($rel.tag) installer during download"; $downloadBlocked = $true }
        else { Note "INFO Defender scan: $($rel.tag) installer download failed: $($_.Exception.Message)" }
      }
      if (Test-Path $published) {
        if ($st -and $st.RealTimeProtectionEnabled) { Note "INFO published $($rel.tag) installer downloaded under active real-time protection without interference" }
        else { Note "INFO published $($rel.tag) installer downloaded; NOTE real-time protection is OFF on this runner, so the download proves nothing" }
        $hash = (Get-FileHash $published -Algorithm SHA256).Hash.ToLower()
        Note "INFO published release $($rel.tag); sha256=$hash matches-release=$($hash -eq $rel.sha)"
        $scanTargets += @{ label = "published $($rel.tag) installer"; path = $published }
      } elseif (-not $downloadBlocked) { Note "INFO Defender scan: $($rel.tag) installer download failed silently" }
    }
    foreach ($t in $scanTargets) {
      $windowStart = Get-Date
      $scanError = $null
      try { Start-MpScan -ScanType CustomScan -ScanPath $t.path } catch { $scanError = $_.Exception.Message }
      Start-Sleep -Seconds 2
      $newThreats = @(Get-MpThreatDetection -ErrorAction SilentlyContinue | Where-Object { $_.InitialDetectionTime -ge $windowStart })
      if ($scanError) { Note "FAIL Defender Start-MpScan reported on $($t.label): $scanError" }
      elseif ($newThreats.Count -gt 0) {
        foreach ($th in $newThreats) { Note "FAIL Defender threat during scan of $($t.label): threatID=$($th.ThreatID) resources=$($th.Resources -join '; ')" }
      } else { Note "PASS Defender Start-MpScan custom scan completed with no detection on $($t.label)" }
    }
  }
} catch { Note "SKIP Defender scan error: $($_.Exception.Message)" }
