param(
  [ValidateSet("export-fixture", "validate-artifact", "evaluate")]
  [string] $Mode = "export-fixture",
  [switch] $DryRun
)

$command = switch ($Mode) {
  "validate-artifact" { "ml-validate-artifact" }
  "evaluate" { "ml-evaluate" }
  default { "ml-export-fixture" }
}

& (Join-Path $PSScriptRoot "grandedge-dev.ps1") $command -DryRun:$DryRun
