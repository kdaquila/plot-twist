# Startup budget check (SC-003): the release app must show its window and serve the local
# API within 2 seconds. The app exits by itself when ready (PLOT_TWIST_EXIT_WHEN_READY=1).
param(
    [double]$BudgetSeconds = 2.0,
    [int]$Runs = 3,
    [switch]$SkipBuild
)
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot

if (-not $SkipBuild) {
    Push-Location $root
    # Build tools log progress to stderr, which Windows PowerShell would treat as an error.
    $ErrorActionPreference = "Continue"
    try {
        npm run tauri build -- --no-bundle 2>&1 | Out-Host
        if ($LASTEXITCODE -ne 0) { throw "release build failed" }
    } finally {
        $ErrorActionPreference = "Stop"
        Pop-Location
    }
}

$exe = Join-Path $root "target\release\plot-twist.exe"
if (-not (Test-Path $exe)) { throw "not found: $exe" }

$env:PLOT_TWIST_EXIT_WHEN_READY = "1"
$times = @()
# The first launch warms WebView2 and the disk cache; it is not counted.
for ($i = 0; $i -le $Runs; $i++) {
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    $process = Start-Process -FilePath $exe -PassThru
    if (-not $process.WaitForExit(30000)) {
        $process.Kill()
        throw "plot-twist did not become ready within 30 s"
    }
    $watch.Stop()
    if ($i -gt 0) { $times += $watch.Elapsed.TotalSeconds }
    Write-Host ("run {0}: {1:N2} s{2}" -f $i, $watch.Elapsed.TotalSeconds, $(if ($i -eq 0) { " (warm-up)" } else { "" }))
}
Remove-Item Env:PLOT_TWIST_EXIT_WHEN_READY

$median = ($times | Sort-Object)[[int][math]::Floor($times.Count / 2)]
Write-Host ("median startup: {0:N2} s (budget {1:N2} s)" -f $median, $BudgetSeconds)
if ($median -gt $BudgetSeconds) {
    throw ("startup {0:N2} s exceeds the {1:N2} s budget" -f $median, $BudgetSeconds)
}
