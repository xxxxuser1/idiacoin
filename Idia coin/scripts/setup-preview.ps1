# PowerShell setup script for preview environment
# This script uses docker-compose exec to run idia-cli commands inside the idia-node container.

$projectRoot = Split-Path $PSScriptRoot -Parent
$composeFile = Join-Path $projectRoot 'docker-compose.preview.yml'
Write-Host "Using compose file: $composeFile" -ForegroundColor DarkCyan

function Exec-InNode {
    param(
        [string[]]$Args
    )
    $cmd = @('exec', 'idia-node') + $Args
    & docker-compose -f "$composeFile" @cmd
}

Write-Host "Generating test accounts..." -ForegroundColor Yellow
$alice = (Exec-InNode './idia-cli' 'account' 'generate' | Out-String).Trim()
$bob = (Exec-InNode './idia-cli' 'account' 'generate' | Out-String).Trim()
$charlie = (Exec-InNode './idia-cli' 'account' 'generate' | Out-String).Trim()

Write-Host "Accounts created:" -ForegroundColor Green
Write-Host "  Alice: $alice"
Write-Host "  Bob: $bob"
Write-Host "  Charlie: $charlie"

Write-Host "Funding test accounts with initial balance..." -ForegroundColor Yellow
Exec-InNode './idia-cli' '--testnet' 'faucet' 'send' $alice '1000000'
Exec-InNode './idia-cli' '--testnet' 'faucet' 'send' $bob '1000000'
Exec-InNode './idia-cli' '--testnet' 'faucet' 'send' $charlie '1000000'

Write-Host "Setting up test liquidity pools..." -ForegroundColor Yellow
Exec-InNode './idia-cli' '--testnet' 'dex' 'create-pool' 'IDIA/ETH'
Exec-InNode './idia-cli' '--testnet' 'dex' 'create-pool' 'IDIA/USDT'

Write-Host "Adding initial liquidity..." -ForegroundColor Yellow
Exec-InNode './idia-cli' '--testnet' 'dex' 'add-liquidity' '--pool' 'IDIA/ETH' '--amount-a' '100000' '--amount-b' '50' '--account' $alice
Exec-InNode './idia-cli' '--testnet' 'dex' 'add-liquidity' '--pool' 'IDIA/USDT' '--amount-a' '100000' '--amount-b' '100000' '--account' $bob

Write-Host "Creating test transactions..." -ForegroundColor Yellow
Exec-InNode './idia-cli' '--testnet' 'tx' 'send' '--from' $alice '--to' $bob '--amount' '1000' '--privacy-level' 'high'
Exec-InNode './idia-cli' '--testnet' 'tx' 'send' '--from' $bob '--to' $charlie '--amount' '500' '--privacy-level' 'medium'

Write-Host "Setting up staking..." -ForegroundColor Yellow
Exec-InNode './idia-cli' '--testnet' 'stake' '--amount' '50000' '--lock-period' '30' '--account' $charlie

Write-Host "Creating test bridge transaction..." -ForegroundColor Yellow
Exec-InNode './idia-cli' '--testnet' 'bridge' '--to' 'ethereum' '--amount' '1000' '--account' $alice

Write-Host "Preview test data initialization complete." -ForegroundColor Green
Write-Host "Test accounts:"
Write-Host "  Alice: $alice"
Write-Host "  Bob:   $bob"
Write-Host "  Charlie: $charlie"
