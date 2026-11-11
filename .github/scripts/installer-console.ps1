# CI-only: observe visible native console windows while NSIS performs cleanup.
# Sampling is best effort, not proof that a window shorter than 10ms cannot flash.
Add-Type -TypeDefinition @"
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
public static class InstallerConsoleWatch {
  delegate bool EnumProc(IntPtr h, IntPtr p);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr p);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  static readonly object gate = new object();
  static HashSet<long> baseline;
  static List<string> found;
  static Thread worker;
  static volatile bool running;
  static int samples;
  static HashSet<long> Visible() {
    var set = new HashSet<long>();
    EnumWindows((h,p) => {
      var name = new StringBuilder(128);
      GetClassNameW(h,name,name.Capacity);
      if (name.ToString() == "ConsoleWindowClass" && IsWindowVisible(h)) set.Add(h.ToInt64());
      return true;
    },IntPtr.Zero);
    return set;
  }
  public static void Start() {
    if (running) throw new InvalidOperationException("Watch already running");
    baseline=Visible(); found=new List<string>(); samples=0; running=true;
    worker=new Thread(() => {
      while (running) {
        foreach (long h in Visible()) if (!baseline.Contains(h)) {
          uint pid; GetWindowThreadProcessId(new IntPtr(h),out pid);
          lock (gate) { found.Add(DateTime.UtcNow.ToString("o")+" hwnd="+h+" pid="+pid); }
        }
        Interlocked.Increment(ref samples);
        Thread.Sleep(10);
      }
    });
    worker.IsBackground=true; worker.Start();
  }
  public static string[] Stop() {
    running=false; if (worker != null) worker.Join();
    lock (gate) { return found.ToArray(); }
  }
  public static int Samples() { return samples; }
}
"@

function Invoke-ObservedInstaller([string]$Path, [string]$Phase) {
  [InstallerConsoleWatch]::Start()
  $events = @()
  try {
    $process = Start-Process -FilePath $Path -ArgumentList '/S' -PassThru
    if (-not $process.WaitForExit(120000)) { throw "NSIS $Phase exceeded 120s" }
    # NSIS uninstall can hand off to a temporary child. Keep observing after
    # the launcher exits, through cleanup and temporary-process shutdown.
    if ($Phase -eq 'uninstall') { Start-Sleep -Seconds 5 }
    else { Start-Sleep -Milliseconds 500 }
    if ($process.ExitCode -ne 0) { throw "NSIS $Phase exit $($process.ExitCode)" }
  } finally {
    $events = @([InstallerConsoleWatch]::Stop())
    $count = [InstallerConsoleWatch]::Samples()
    "INFO $Phase console observation: $count samples at requested 10ms interval, visible ConsoleWindowClass only; shorter windows and other window classes can be missed" | Add-Content installer-console-results.txt
    if ($events.Count) { $events | Add-Content installer-console-results.txt }
  }
  if ($events.Count) { throw "Visible console window observed during $Phase" }
  "PASS $Phase no new visible native console window observed" | Add-Content installer-console-results.txt
}
