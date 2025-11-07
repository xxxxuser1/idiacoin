# Start preview environment
Write-Host "Starting Idia Coin preview environment..." -ForegroundColor Green

# Check Docker
if (-not (Get-Command docker -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Docker is not installed!" -ForegroundColor Red
    exit 1
}

# Build and start services
Write-Host "Building and starting services..." -ForegroundColor Yellow

# Resolve compose file path relative to this script so paths with spaces work
$projectRoot = Split-Path $PSScriptRoot -Parent
$composeFile = Join-Path $projectRoot 'docker-compose.preview.yml'

Write-Host "Using compose file: $composeFile" -ForegroundColor DarkCyan
& docker-compose -f "$composeFile" up --build -d

# Wait for node to be ready
Write-Host "Waiting for node to be ready..." -ForegroundColor Yellow
$attempts = 0
$maxAttempts = 30
do {
    $attempts++
    Start-Sleep -Seconds 2
    try {
        $response = Invoke-WebRequest "http://localhost:8081/health" -UseBasicParsing
        if ($response.StatusCode -eq 200) {
            break
        }
    }
    catch {
        if ($attempts -eq $maxAttempts) {
            Write-Host "Error: Node failed to start!" -ForegroundColor Red
            exit 1
        }
        Write-Host "Waiting for node... ($attempts/$maxAttempts)" -ForegroundColor Yellow
    }
} while ($true)

# Initialize test data
Write-Host "Initializing test data..." -ForegroundColor Yellow
# Attempt to run the setup script inside the idia-node container (may require the script to be present in the container).
try {
    & docker-compose -f "$composeFile" exec idia-node /bin/sh -c '/scripts/setup-preview.sh'
}
catch {
    Write-Host "Warning: Could not run setup script inside container. Falling back to host PowerShell setup script..." -ForegroundColor Yellow
    $hostSetup = Join-Path $projectRoot 'scripts\setup-preview.ps1'
    if (Test-Path $hostSetup) {
        Write-Host "Running host setup script: $hostSetup" -ForegroundColor DarkCyan
        & "$hostSetup"
    }
    else {
        Write-Host "Host setup script not found at: $hostSetup" -ForegroundColor Red
        Write-Host "You can run the Unix setup script inside the container manually:" -ForegroundColor Yellow
        Write-Host "  docker-compose -f \"$composeFile\" exec idia-node /bin/sh -c '/scripts/setup-preview.sh'" -ForegroundColor Yellow
    }
}

Write-Host "`nPreview environment is ready!" -ForegroundColor Green
Write-Host "Access points:"
Write-Host "- Explorer: http://localhost:3000"
Write-Host "- Wallet UI: http://localhost:3001"
Write-Host "- DEX UI: http://localhost:3002"
Write-Host "- Metrics: http://localhost:3003"

Write-Host "`nTo stop the preview:" -ForegroundColor Yellow
Write-Host "& docker-compose -f \"$composeFile\" down"