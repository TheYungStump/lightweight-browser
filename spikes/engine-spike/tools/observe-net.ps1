# Non-admin network observation for the Phase 0b telemetry check.
# Runs the spike once per variant and, every 250 ms, records the remote TCP
# endpoints of the msedgewebview2.exe processes that belong to that run's
# profile (matched by the profile folder in their command line), and snapshots
# the DNS client cache to map IPs to host names. Limits: no UDP/QUIC remote
# addresses (Get-NetUDPEndpoint has none), no payloads, and DNS names only for
# IPs that are in the Windows DNS cache (Chromium often resolves with its own
# resolver, so many names are missing; forward lookups fill them in). Elevated pktmon/DNS-log steps are separate
# (docs/testing/phase-0b.md).
param(
  [string]$Exe = "$PSScriptRoot\..\target\release\engine-spike.exe",
  [string]$Script = "$PSScriptRoot\..\scripts\telemetry.txt",
  [string]$OutDir = "$env:LOCALAPPDATA\lwb-spike\net",
  [string[]]$Variants = @("A-default-fresh", "A-default-reused", "B-crash-upload", "C-no-reputation", "D-flag-no-smartscreen", "E-all-controls")
)
New-Item -ItemType Directory -Force $OutDir | Out-Null
$runId = Get-Date -Format "yyyyMMdd-HHmmss"

function Get-VariantArgs($v) {
  switch -Wildcard ($v) {
    "A-default-fresh"       { return @("--tag", "net-A-$runId", "--args", '""') }
    "A-default-reused"      { return @("--tag", "net-A-$runId", "--args", '""') }
    "B-crash-upload"        { return @("--tag", "net-B-$runId", "--args", '""', "--no-crash-local") }
    "C-no-reputation"       { return @("--tag", "net-C-$runId", "--args", '""', "--no-reputation") }
    "D-flag-no-smartscreen" { return @("--tag", "net-D-$runId", "--args", '"--disable-features=msSmartScreenProtection"') }
    "E-all-controls"        { return @("--tag", "net-E-$runId", "--args", '"--disable-features=msSmartScreenProtection"', "--no-reputation") }
    "F-all-plus-ecs-policy" { return @("--tag", "net-F-$runId", "--args", '"--disable-features=msSmartScreenProtection"', "--no-reputation") }
  }
}

$summary = @()
foreach ($v in $Variants) {
  $va = Get-VariantArgs $v
  $tag = $va[1]
  $argList = @("--no-chrome", "--views", "1", "--url", "about:blank", "--script", "`"$Script`"") + $va
  $start = Get-Date
  $p = Start-Process -FilePath $Exe -ArgumentList $argList -PassThru -WindowStyle Minimized
  $seen = @{}
  $dns = @{}
  while (-not $p.HasExited) {
    $pids = @(Get-CimInstance Win32_Process -Filter "Name='msedgewebview2.exe'" |
      Where-Object { $_.CommandLine -like "*profile-$tag*" } | ForEach-Object { [int]$_.ProcessId })
    if ($pids.Count -gt 0) {
      Get-NetTCPConnection -ErrorAction SilentlyContinue | Where-Object { $pids -contains $_.OwningProcess -and $_.RemoteAddress -notin @("0.0.0.0", "::", "127.0.0.1", "::1") } |
        ForEach-Object {
          $k = "$($_.RemoteAddress):$($_.RemotePort)"
          if (-not $seen.ContainsKey($k)) { $seen[$k] = [int]((Get-Date) - $start).TotalSeconds }
        }
    }
    Get-DnsClientCache -ErrorAction SilentlyContinue | Where-Object { $_.Data } | ForEach-Object { $dns[$_.Data] = $_.Entry }
    Start-Sleep -Milliseconds 250
  }
  $rows = $seen.GetEnumerator() | Sort-Object Value | ForEach-Object {
    $ip = ($_.Key -replace ':\d+$', '')
    [pscustomobject]@{ variant = $v; t_s = $_.Value; endpoint = $_.Key; host = $dns[$ip] }
  }
  $rows | Export-Csv -NoTypeInformation -Path "$OutDir\$runId-$v.csv"
  $summary += $rows
  Start-Sleep -Seconds 3
}
$summary | Export-Csv -NoTypeInformation -Path "$OutDir\$runId-all.csv"
$summary | Format-Table -AutoSize | Out-String -Width 200
