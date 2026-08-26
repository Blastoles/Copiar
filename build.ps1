# Script de Build e Incremento Automático de Versão
param (
    [ValidateSet("patch", "minor", "major")]
    [string]$IncrementType = "patch"
)

$ErrorActionPreference = "Stop"

# Helper para gravar UTF-8 sem BOM
function Set-Utf8NoBomContent($path, $text) {
    $utf8NoBom = New-Object System.Text.UTF8Encoding($false)
    [System.IO.File]::WriteAllText($path, $text, $utf8NoBom)
}

function Get-Utf8Content($path) {
    return [System.IO.File]::ReadAllText($path, [System.Text.Encoding]::UTF8)
}

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "   COPIAR - BUILD E VERSIONAMENTO AUTO    " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

$packageJsonPath = Join-Path $PSScriptRoot "package.json"
$cargoTomlPath = Join-Path $PSScriptRoot "src-tauri\Cargo.toml"
$tauriConfPath = Join-Path $PSScriptRoot "src-tauri\tauri.conf.json"
$indexHtmlPath = Join-Path $PSScriptRoot "ui\index.html"

if (-not (Test-Path $packageJsonPath)) {
    Write-Error "package.json não encontrado!"
}

# 1. Ler versão atual
$packageContent = Get-Utf8Content $packageJsonPath
if ($packageContent -match '"version"\s*:\s*"([^"]+)"') {
    $currentVer = $matches[1]
} else {
    $currentVer = "1.0.0"
}
Write-Host "[*] Versão Atual: $currentVer" -ForegroundColor Yellow

# Parsear versão semântica (X.Y.Z)
$parts = $currentVer.Split('.')
[int]$major = if ($parts.Length -ge 1) { [int]$parts[0] } else { 1 }
[int]$minor = if ($parts.Length -ge 2) { [int]$parts[1] } else { 0 }
[int]$patch = if ($parts.Length -ge 3) { [int]$parts[2] } else { 0 }

switch ($IncrementType) {
    "major" { $major++; $minor = 0; $patch = 0 }
    "minor" { $minor++; $patch = 0 }
    "patch" { $patch++ }
}

$newVersion = "$major.$minor.$patch"
$displayVersion = "v$major.$minor.$patch"
Write-Host "[+] Nova Versão: $newVersion" -ForegroundColor Green

# 2. Atualizar package.json
$packageContent = [regex]::Replace($packageContent, '"version"\s*:\s*"[^"]+"', "`"version`": `"$newVersion`"", 1)
Set-Utf8NoBomContent $packageJsonPath $packageContent
Write-Host "  -> Atualizado: package.json" -ForegroundColor Gray

# 3. Atualizar src-tauri/Cargo.toml
if (Test-Path $cargoTomlPath) {
    $cargoContent = Get-Utf8Content $cargoTomlPath
    $cargoContent = [regex]::Replace($cargoContent, '(?m)^version\s*=\s*"[^"]+"', "version = `"$newVersion`"", 1)
    Set-Utf8NoBomContent $cargoTomlPath $cargoContent
    Write-Host "  -> Atualizado: src-tauri/Cargo.toml" -ForegroundColor Gray
}

# 4. Atualizar src-tauri/tauri.conf.json
if (Test-Path $tauriConfPath) {
    $tauriContent = Get-Utf8Content $tauriConfPath
    $tauriContent = [regex]::Replace($tauriContent, '"version"\s*:\s*"[^"]+"', "`"version`": `"$newVersion`"", 1)
    Set-Utf8NoBomContent $tauriConfPath $tauriContent
    Write-Host "  -> Atualizado: src-tauri/tauri.conf.json" -ForegroundColor Gray
}

# 5. Atualizar ui/index.html
if (Test-Path $indexHtmlPath) {
    $htmlContent = Get-Utf8Content $indexHtmlPath
    $htmlContent = [regex]::Replace($htmlContent, '<span>v[^<]+</span>', "<span>$displayVersion</span>", 1)
    Set-Utf8NoBomContent $indexHtmlPath $htmlContent
    Write-Host "  -> Atualizado: ui/index.html" -ForegroundColor Gray
}

# 6. Executar Build Completo
Write-Host "`n[*] Iniciando compilação de produção com Tauri CLI..." -ForegroundColor Cyan

if (-not (Test-Path (Join-Path $PSScriptRoot "node_modules"))) {
    Write-Host "[*] Instalando dependências npm..." -ForegroundColor Yellow
    npm install
}

# Limitar threads do Cargo para evitar travamento do sistema
$cores = [Math]::Max(1, [Environment]::ProcessorCount - 2)
$env:CARGO_BUILD_JOBS = $cores
Write-Host "[*] Compilando com $cores threads (evitando travamento)..." -ForegroundColor Yellow

npx @tauri-apps/cli build

if ($LASTEXITCODE -eq 0) {
    Write-Host "`n==========================================" -ForegroundColor Green
    Write-Host "   BUILD $newVersion CONCLUÍDO COM SUCESSO!   " -ForegroundColor Green
    Write-Host "==========================================" -ForegroundColor Green
    Write-Host "Executável / Instalador gerado em:" -ForegroundColor White
    Write-Host "  src-tauri\target\release\" -ForegroundColor Yellow
    Write-Host "  src-tauri\target\release\bundle\" -ForegroundColor Yellow
} else {
    Write-Error "Falha durante o build do Tauri!"
}
