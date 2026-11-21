# Phase 0b telemetry check, ELEVATED part (run in an Administrator PowerShell).
# Adds what the non-admin observer can't see: DNS queries (incl. ones made for
# QUIC/UDP and very short connections) and a packet capture of the whole run.
# Close other browsers and chat apps first so the machine is quiet.
#
# What it does:
#   1. turns on the DNS-Client Operational event log (turned back off at the end)
#   2. starts a pktmon packet capture
#   3. runs observe-net.ps1 (all control variants, ~9 minutes)
#   4. sets the WebView2 policy ExperimentationAndConfigurationServiceControl = 0
#      (RestrictedMode) for one extra run (variant F), then restores it
#   5. stops the capture, exports DNS query names seen during the run, and
#      converts the capture to a .pcapng you can open in Wireshark if needed
# Output: %LOCALAPPDATA%\lwb-spike\net-elevated\<timestamp>\
$ErrorActionPreference = "Stop"
$id = [Security.Principal.WindowsIdentity]::GetCurrent()
if (-not (New-Object Security.Principal.WindowsPrincipal $id).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  throw "Run this from an Administrator PowerShell."
}
$out = Join-Path $env:LOCALAPPDATA ("lwb-spike\net-elevated\" + (Get-Date -Format "yyyyMMdd-HHmmss"))
New-Item -ItemType Directory -Force $out | Out-Null
$dnsLog = "Microsoft-Windows-DNS-Client/Operational"
$wasEnabled = (Get-WinEvent -ListLog $dnsLog).IsEnabled

wevtutil sl $dnsLog /e:true
$start = Get-Date
pktmon stop 2>$null | Out-Null
pktmon filter remove | Out-Null
pktmon start --capture --pkt-size 256 --file-name (Join-Path $out "capture.etl") | Out-Null
$polKey = "HKLM:\SOFTWARE\Policies\Microsoft\Edge\WebView2"
$polName = "ExperimentationAndConfigurationServiceControl"
$hadKey = Test-Path $polKey
$oldVal = if ($hadKey) { (Get-ItemProperty $polKey -Name $polName -ErrorAction SilentlyContinue).$polName } else { $null }
try {
  & (Join-Path $PSScriptRoot "observe-net.ps1") -OutDir $out
  New-Item -Path $polKey -Force | Out-Null
  Set-ItemProperty -Path $polKey -Name $polName -Type DWord -Value 0
  & (Join-Path $PSScriptRoot "observe-net.ps1") -OutDir $out -Variants @("F-all-plus-ecs-policy")
} finally {
  if ($null -ne $oldVal) { Set-ItemProperty -Path $polKey -Name $polName -Type DWord -Value $oldVal }
  elseif (Test-Path $polKey) {
    Remove-ItemProperty -Path $polKey -Name $polName -ErrorAction SilentlyContinue
    if (-not $hadKey) { Remove-Item -Path $polKey -ErrorAction SilentlyContinue }
  }
  pktmon stop | Out-Null
  $end = Get-Date
  Get-WinEvent -FilterHashtable @{ LogName = $dnsLog; StartTime = $start; EndTime = $end; Id = 3008 } -ErrorAction SilentlyContinue |
    ForEach-Object { [pscustomobject]@{ time = $_.TimeCreated; query = $_.Properties[0].Value; pid = $_.ProcessId } } |
    Export-Csv -NoTypeInformation (Join-Path $out "dns-queries.csv")
  if (-not $wasEnabled) { wevtutil sl $dnsLog /e:false }
  pktmon etl2pcap (Join-Path $out "capture.etl") --out (Join-Path $out "capture.pcapng") | Out-Null
  Write-Host "Done. Results in $out (observe-net CSVs, dns-queries.csv, capture.pcapng)."
}
