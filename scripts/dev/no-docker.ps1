param([switch] $DryRun)
& (Join-Path $PSScriptRoot "grandedge-dev.ps1") "no-docker" @PSBoundParameters
