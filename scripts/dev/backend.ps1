param([switch] $DryRun)
& (Join-Path $PSScriptRoot "grandedge-dev.ps1") "backend" @PSBoundParameters
