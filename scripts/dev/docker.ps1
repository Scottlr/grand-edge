param(
  [ValidateSet("up", "down")]
  [string] $Mode = "up",
  [switch] $DryRun
)

$command = if ($Mode -eq "down") { "docker-down" } else { "docker-up" }
& (Join-Path $PSScriptRoot "grandedge-dev.ps1") $command -DryRun:$DryRun
