#!/bin/bash
# Solive Delivery Verification Script
# Fetches worker deliveries, runs spec checks, and helps approve/reject

set -e

# Job definitions: name, intent_id, spec_hash
declare -A JOBS=(
    ["SPATIAL_INDEXING"]="qTtZ4FwQcTCiFHumjqVKBLWiVU5qXRp5CVLqZaheUJ3|99ff47155df1c3a8874ddb8c8b43337f529b3bd9d7d547239531789c7f2b2883"
    ["MEMORY_POOLING"]="3JPTpwnPz3zaSBkgheCjwxQHQFjE3EX7N3529VWiKJbs|a41ce085a9ee89a6860a54308c73296dbad9743514e9d3d2511f9031643cfd53"
    ["GPU_BATCHING"]="He52n3JgkNCMCGEkVdzX55kchLsYnHhwjVwMyMhp6Co6|d5b1f67862345d9a8ecc750e26929ddbc619a4ff7003e0c58efe5176119fb8fd"
    ["CONFIG_SYSTEM"]="8PT8NT85SLBDATYPkBM58gMQC43uEoXB8YQL2GHSAj72|61d963c47c05191e511f4fa3cc6b44dc92e7eb928b2f1d594f84fa89524fad88"
    ["PREFAB_SYSTEM"]="8vfjG2MfNjPUwvaxu2z9qrqSnqjUapZYsLink7xPmxSA|dc4730d9a427449e40d2a6e1ae9c55c86fcfbea29d76c689da65659d814d76b7"
    ["DEBUG_PROFILER"]="AVkBPYASsYi4ZEp7ggJRjoAgnUXUMdyUn3AS9FN4cApk|a5660d75d2b936cf074af44466262e52e35b454930a2225eb132aec75c107e60"
)

echo "=== Solive Job Verification Tool ==="
echo ""
echo "Available jobs to verify:"
echo "  1. SPATIAL_INDEXING  - Quadtree spatial indexing"
echo "  2. MEMORY_POOLING    - Entity object pooling"
echo "  3. GPU_BATCHING      - GPU sprite instancing"
echo "  4. CONFIG_SYSTEM     - YAML game config"
echo "  5. PREFAB_SYSTEM     - Entity template system"
echo "  6. DEBUG_PROFILER    - In-game debug tools"
echo ""
read -p "Enter job name (or 'all' to check all): " JOB_INPUT

# Function to check a job
check_job() {
    local job_name="$1"
    local job_data="${JOBS[$job_name]}"

    if [ -z "$job_data" ]; then
        echo "❌ Unknown job: $job_name"
        return 1
    fi

    IFS='|' read -r intent_id spec_hash <<< "$job_data"

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "🔍 Checking: $job_name"
    echo "   Intent ID: $intent_id"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Check for delivery
    echo "1️⃣  Looking for delivery announcement..."
    if solive wallet check-delivery --intent "$intent_id" 2>/dev/null | grep -q "delivered"; then
        echo "   ✅ Work has been delivered!"

        # Fetch work
        echo ""
        echo "2️⃣  Fetching deliverable..."
        WORK_DIR=$(mktemp -d)
        solive wallet fetch-work --intent "$intent_id" --output "$WORK_DIR"
        echo "   ✅ Saved to: $WORK_DIR"

        # Run judge check
        echo ""
        echo "3️⃣  Running spec verification..."
        SPEC_FILE="SOLIVE_SPEC_$(echo $job_name | sed 's/_//'| cut -c1-2)_*.md"

        if [ -f "$SPEC_FILE" ]; then
            if solive judge "$WORK_DIR" 2>&1 | tee judge_output.txt; then
                echo "   ✅ SPEC VERIFICATION PASSED!"
                echo ""
                echo "═══════════════════════════════════════════════════════════"
                echo "✨ READY TO APPROVE ✨"
                echo "═══════════════════════════════════════════════════════════"
                echo "Run to release payment:"
                echo ""
                echo "  solive wallet attest \\\"
                echo "    --intent '$intent_id' \\\"
                echo "    --verdict approve \\\"
                echo "    --feedback 'Verified: spec tests pass'"
                echo ""
            else
                echo "   ❌ SPEC VERIFICATION FAILED"
                echo ""
                echo "═══════════════════════════════════════════════════════════"
                echo "⚠️  ISSUES FOUND ⚠️"
                echo "═══════════════════════════════════════════════════════════"
                echo ""
                cat judge_output.txt
                echo ""
                echo "Run to request revision:"
                echo ""
                echo "  solive wallet refute \\\"
                echo "    --intent '$intent_id' \\\"
                echo "    --feedback 'Fix: [specific issue from above]'"
                echo ""
            fi
        else
            echo "   ⚠️  Spec file not found: $SPEC_FILE"
            echo "   Manual verification needed. Work directory: $WORK_DIR"
        fi

        echo ""
        read -p "Keep work directory for inspection? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$WORK_DIR"
        else
            echo "   📁 Preserved at: $WORK_DIR"
        fi
    else
        echo "   ⏳ No delivery yet. Check back later."
        echo ""
        echo "   Next steps:"
        echo "   - Worker claims job with: solive wallet claim --intent '$intent_id'"
        echo "   - Worker implements + tests locally"
        echo "   - Worker submits with: solive wallet deliver --intent '$intent_id' --url <github-branch>"
        echo ""
    fi
}

# Process input
if [ "$JOB_INPUT" = "all" ]; then
    for job_name in "${!JOBS[@]}"; do
        check_job "$job_name"
    done
else
    check_job "${JOB_INPUT^^}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "💡 Tip: Run this script again to check for new deliveries"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
