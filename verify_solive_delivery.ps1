# Solive Delivery Verification Script (Windows PowerShell)
# Checks worker deliveries, runs spec verifications, and helps approve/reject

$ErrorActionPreference = "Stop"

# Job definitions
$Jobs = @{
    "SPATIAL_INDEXING" = @{
        IntentId = "qTtZ4FwQcTCiFHumjqVKBLWiVU5qXRp5CVLqZaheUJ3"
        SpecHash = "99ff47155df1c3a8874ddb8c8b43337f529b3bd9d7d547239531789c7f2b2883"
        SpecFile = "SOLIVE_SPEC_01_SPATIAL_INDEXING.md"
        Description = "Quadtree spatial indexing"
    }
    "MEMORY_POOLING" = @{
        IntentId = "3JPTpwnPz3zaSBkgheCjwxQHQFjE3EX7N3529VWiKJbs"
        SpecHash = "a41ce085a9ee89a6860a54308c73296dbad9743514e9d3d2511f9031643cfd53"
        SpecFile = "SOLIVE_SPEC_02_MEMORY_POOLING.md"
        Description = "Entity object pooling"
    }
    "GPU_BATCHING" = @{
        IntentId = "He52n3JgkNCMCGEkVdzX55kchLsYnHhwjVwMyMhp6Co6"
        SpecHash = "d5b1f67862345d9a8ecc750e26929ddbc619a4ff7003e0c58efe5176119fb8fd"
        SpecFile = "SOLIVE_SPEC_03_GPU_BATCHING.md"
        Description = "GPU sprite instancing"
    }
    "CONFIG_SYSTEM" = @{
        IntentId = "8PT8NT85SLBDATYPkBM58gMQC43uEoXB8YQL2GHSAj72"
        SpecHash = "61d963c47c05191e511f4fa3cc6b44dc92e7eb928b2f1d594f84fa89524fad88"
        SpecFile = "SOLIVE_SPEC_04_CONFIG_SYSTEM.md"
        Description = "YAML game config"
    }
    "PREFAB_SYSTEM" = @{
        IntentId = "8vfjG2MfNjPUwvaxu2z9qrqSnqjUapZYsLink7xPmxSA"
        SpecHash = "dc4730d9a427449e40d2a6e1ae9c55c86fcfbea29d76c689da65659d814d76b7"
        SpecFile = "SOLIVE_SPEC_05_PREFAB_SYSTEM.md"
        Description = "Entity template system"
    }
    "DEBUG_PROFILER" = @{
        IntentId = "AVkBPYASsYi4ZEp7ggJRjoAgnUXUMdyUn3AS9FN4cApk"
        SpecHash = "a5660d75d2b936cf074af44466262e52e35b454930a2225eb132aec75c107e60"
        SpecFile = "SOLIVE_SPEC_06_DEBUG_PROFILER.md"
        Description = "In-game debug tools"
    }
}

function Check-JobDelivery {
    param([string]$JobName)

    $job = $Jobs[$JobName]
    if (-not $job) {
        Write-Host "❌ Unknown job: $JobName" -ForegroundColor Red
        return
    }

    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "🔍 Checking: $JobName" -ForegroundColor Cyan
    Write-Host "   Description: $($job.Description)" -ForegroundColor Gray
    Write-Host "   Intent ID: $($job.IntentId)" -ForegroundColor Gray
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""

    # Check for delivery
    Write-Host "1️⃣  Looking for delivery announcement..." -ForegroundColor Yellow

    try {
        $delivery = solive wallet check-delivery --intent $job.IntentId 2>&1

        if ($delivery -match "delivered|announced") {
            Write-Host "   ✅ Work has been delivered!" -ForegroundColor Green

            # Fetch work
            Write-Host ""
            Write-Host "2️⃣  Fetching deliverable..." -ForegroundColor Yellow

            $workDir = New-Item -ItemType Directory -Path (Join-Path $env:TEMP "solive_work_$JobName") -Force
            Write-Host "   Downloading to: $($workDir.FullName)" -ForegroundColor Gray

            solive wallet fetch-work --intent $job.IntentId --output $workDir.FullName 2>&1 | Out-Null
            Write-Host "   ✅ Delivery fetched" -ForegroundColor Green

            # Show what was delivered
            Write-Host ""
            Write-Host "3️⃣  Deliverables:" -ForegroundColor Yellow
            Get-ChildItem -Path $workDir.FullName -Recurse -File | ForEach-Object {
                Write-Host "   📄 $($_.FullName.Replace($workDir.FullName, ''))" -ForegroundColor Gray
            }

            # Judge check instructions
            Write-Host ""
            Write-Host "4️⃣  Verification Steps:" -ForegroundColor Yellow
            Write-Host "   a) Review the code in: $($workDir.FullName)" -ForegroundColor Gray
            Write-Host "   b) Check against spec: $($job.SpecFile)" -ForegroundColor Gray
            Write-Host "   c) Run: solive judge '$($workDir.FullName)'" -ForegroundColor Gray
            Write-Host ""

            # Provide approval commands
            Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Green
            Write-Host "✅ IF TESTS PASS, approve with:" -ForegroundColor Green
            Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Green
            Write-Host ""
            Write-Host "solive wallet attest ``" -ForegroundColor Cyan
            Write-Host "  --intent '$($job.IntentId)' ``" -ForegroundColor Cyan
            Write-Host "  --verdict approve ``" -ForegroundColor Cyan
            Write-Host "  --feedback 'Verified: all spec tests pass'" -ForegroundColor Cyan
            Write-Host ""

            Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Red
            Write-Host "❌ IF TESTS FAIL, request revision with:" -ForegroundColor Red
            Write-Host "═══════════════════════════════════════════════════════════" -ForegroundColor Red
            Write-Host ""
            Write-Host "solive wallet refute ``" -ForegroundColor Cyan
            Write-Host "  --intent '$($job.IntentId)' ``" -ForegroundColor Cyan
            Write-Host "  --feedback 'Fix: [describe what failed]'" -ForegroundColor Cyan
            Write-Host ""

            Write-Host "📁 Work directory kept at: $($workDir.FullName)" -ForegroundColor Cyan

        } else {
            Write-Host "   ⏳ No delivery yet" -ForegroundColor Yellow
            Write-Host ""
            Write-Host "   Status:" -ForegroundColor Gray
            Write-Host "   - Worker should claim with: solive wallet claim --intent '$($job.IntentId)'" -ForegroundColor Gray
            Write-Host "   - Worker implements and tests locally" -ForegroundColor Gray
            Write-Host "   - Worker submits with: solive wallet deliver --intent '$($job.IntentId)' --url <github-url>" -ForegroundColor Gray
        }
    } catch {
        Write-Host "   ⚠️  Error checking delivery: $_" -ForegroundColor Yellow
        Write-Host "   Make sure solive CLI is installed and wallet is set up" -ForegroundColor Gray
    }
}

# Main
Write-Host ""
Write-Host "╔════════════════════════════════════════════════════════════╗" -ForegroundColor Cyan
Write-Host "║  Solive Job Delivery Verification Tool                   ║" -ForegroundColor Cyan
Write-Host "╚════════════════════════════════════════════════════════════╝" -ForegroundColor Cyan
Write-Host ""

Write-Host "Available jobs:" -ForegroundColor White
$Jobs.Keys | ForEach-Object { Write-Host "  • $_" -ForegroundColor Gray }
Write-Host ""

$jobInput = Read-Host "Enter job name or 'all' to check all"

if ($jobInput -eq "all") {
    $Jobs.Keys | ForEach-Object {
        Check-JobDelivery $_
    }
} else {
    Check-JobDelivery $jobInput.ToUpper()
}

Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host "💡 Run this script again to check for new deliveries" -ForegroundColor Yellow
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Gray
Write-Host ""
