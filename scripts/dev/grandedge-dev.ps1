param(
  [Parameter(Position = 0)]
  [ValidateSet("doctor", "no-docker", "docker-up", "docker-down", "backend", "frontend", "ingest-once", "ml-export-fixture", "ml-validate-artifact", "ml-evaluate", "urls")]
  [string] $Command = "doctor",

  [switch] $DryRun
)

$ErrorActionPreference = "Stop"
$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$DockerComposeFile = Join-Path $RepoRoot "docker-compose.dev.yml"
$ApiHealthUrl = "http://localhost:3000/health"
$OpenApiUrl = "http://localhost:3000/api/openapi.json"
$FrontendUrl = "http://localhost:5173"

function Write-Section {
  param([string] $Label)
  Write-Host ""
  Write-Host "[$Label]"
}

function Test-CommandAvailable {
  param([string] $Name)
  return $null -ne (Get-Command $Name -ErrorAction SilentlyContinue)
}

function Invoke-Step {
  param(
    [string] $Label,
    [string] $CommandText
  )

  Write-Host "$Label"
  Write-Host "  $CommandText"

  if ($DryRun) {
    return
  }

  & powershell -NoProfile -Command $CommandText
  if ($LASTEXITCODE -ne 0) {
    throw "Command failed: $CommandText"
  }
}

function Show-Urls {
  Write-Section "URLs"
  Write-Host "API health: $ApiHealthUrl"
  Write-Host "OpenAPI:    $OpenApiUrl"
  Write-Host "Frontend:   $FrontendUrl"
}

function Run-Doctor {
  Write-Section "Prerequisites"
  foreach ($tool in @("rustup", "cargo", "node", "npm", "uv", "docker", "psql")) {
    $status = if (Test-CommandAvailable $tool) { "available" } else { "missing" }
    Write-Host "${tool}: $status"
  }

  Write-Section "Environment files"
  foreach ($path in @(
    ".env.example",
    ".env.docker.example",
    "configs/default.toml",
    "configs/local.example.toml",
    "docs/running/no-docker.md",
    "docs/running/docker.md",
    "docs/running/ml-workflow.md",
    "docs/running/troubleshooting.md"
  )) {
    $fullPath = Join-Path $RepoRoot $path
    $status = if (Test-Path $fullPath) { "present" } else { "missing" }
    Write-Host "${path}: $status"
  }

  Write-Section "Typed commands"
  foreach ($commandText in @(
    "cargo run -p grand-edge-xtask -- doctor",
    "cargo run -p grand-edge-xtask -- db migrate",
    "cargo run -p grand-edge-xtask -- server run --profile local",
    "cargo run -p grand-edge-xtask -- ingest latest --profile local",
    "cargo run -p grand-edge-xtask -- analytics export-features --from 2026-01-01 --to 2026-02-01 --out reports/datasets/jan --include-raw-interval-candles",
    "cargo run -p grand-edge-xtask -- model validate --artifact ml/artifacts/fixture",
    "cargo run -p grand-edge-xtask -- model evaluate --strategy gbm_ranker_v1 --version 2026-06-16.1 --artifact ml/artifacts/gbm_ranker_v1/2026-06-16.1"
  )) {
    Write-Host $commandText
  }

  Show-Urls
}

switch ($Command) {
  "doctor" {
    Run-Doctor
  }
  "no-docker" {
    Run-Doctor
    Write-Section "No-Docker Workflow"
    Invoke-Step "Database migrate" "cargo run -p grand-edge-xtask -- db migrate"
    Invoke-Step "Optional ingestion smoke" "cargo run -p grand-edge-xtask -- ingest latest --profile local"
    Invoke-Step "Backend server" "cargo run -p grand-edge-xtask -- server run --profile local"
    Invoke-Step "Frontend dev server" "npm --prefix apps/web run dev"
    Show-Urls
  }
  "docker-up" {
    Write-Section "Docker Compose"
    Invoke-Step "Full stack" "docker compose -f `"$DockerComposeFile`" up --build"
  }
  "docker-down" {
    Write-Section "Docker Compose"
    Invoke-Step "Stop stack" "docker compose -f `"$DockerComposeFile`" down"
  }
  "backend" {
    Invoke-Step "Backend server" "cargo run -p grand-edge-xtask -- server run --profile local"
  }
  "frontend" {
    Invoke-Step "Frontend dev server" "npm --prefix apps/web run dev"
  }
  "ingest-once" {
    Invoke-Step "One ingestion pass" "cargo run -p grand-edge-xtask -- ingest latest --profile local"
  }
  "ml-export-fixture" {
    Invoke-Step "Fixture artifact export" "uv run --project ml python -m grandedge_ml.export --fixture --out ml/artifacts/fixture"
  }
  "ml-validate-artifact" {
    Invoke-Step "Rust artifact validation" "cargo run -p grand-edge-xtask -- model validate --artifact ml/artifacts/fixture"
  }
  "ml-evaluate" {
    Invoke-Step "Rust artifact evaluation" "cargo run -p grand-edge-xtask -- model evaluate --strategy gbm_ranker_v1 --version 2026-06-16.1 --artifact ml/artifacts/fixture"
  }
  "urls" {
    Show-Urls
  }
}
