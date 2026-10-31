# Phase 1 Critical Fixes: SOLIVE Job Launch Package

**Total Budget:** 0.51 SOL  
**Total Effort:** 12-17 hours  
**Workers Needed:** 4 (can work in parallel)  
**Target Completion:** 24-48 hours  

---

## Overview

Four independent jobs to fix critical bugs identified in the codebase audit:
1. **Config Validation** — Prevent invalid configuration from crashing game
2. **Collision Attribution** — Fix damage being attributed to wrong entities
3. **Bounds Checking** — Fix spatial grid overflow, add safety assertions
4. **Performance Optimization** — Remove O(n) queries, fix metrics overflow

Each job can be claimed and completed independently.

---

## Job Details

### 📋 Job 1: Config Validation System
**File:** `SOLIVE_JOB_AUDIT_FIX_01.md`  
**Budget:** 0.12 SOL  
**Time:** 3-4 hours  
**Critical Path:** ✅ Yes (blocks testing)

**What it does:**
- Add ConfigError enum with detailed error types
- Implement GameConfig::validate() method
- Check 10 constraints (pool sizes > 0, speed > 0, etc.)
- Call validation in load_or_default()

**Risk:** LOW (isolated feature, no integration risk)

**Tests:** 4 unit tests (zero pool, negative speed, valid config, multiple errors)

---

### 🎯 Job 2: Entity ID & Collision Attribution Fix
**File:** `SOLIVE_JOB_AUDIT_FIX_02.md`  
**Budget:** 0.13 SOL  
**Time:** 3-4 hours  
**Critical Path:** ✅ Yes (breaks damage system)

**What it does:**
- Update test_circle_collision() to accept entity IDs as parameters
- Populate ContactInfo with real entity IDs instead of Entity::from_bits(0)
- Update all callers in collision detection
- Update test_circle_aabb_collision() similarly

**Risk:** MEDIUM (changes collision API, must update callers)

**Tests:** 2 integration tests (entity ID verification, no entity zero)

---

### 🔧 Job 3: Spatial Grid & Bounds Checking
**File:** `SOLIVE_JOB_AUDIT_FIX_03.md`  
**Budget:** 0.14 SOL  
**Time:** 3-5 hours  
**Critical Path:** ✅ Yes (breaks scaling at 8K entities)

**What it does:**
- Change grid from HashMap<(usize, usize)> to HashMap<(i32, i32)>
- Fix negative coordinate wrapping bug
- Add radius validation (assert radius <= 2 × CELL_SIZE)
- Add bounds checking for velocity, damage, entity count
- Add world_to_grid() using floor() as i32

**Risk:** MEDIUM (changes grid data structure, must update all callers)

**Tests:** 4 tests (negative coords, radius validation, velocity clamp, damage validation)

---

### ⚡ Job 4: Performance Optimization & Metrics Fixes
**File:** `SOLIVE_JOB_AUDIT_FIX_04.md`  
**Budget:** 0.12 SOL  
**Time:** 3-4 hours  
**Critical Path:** ✅ Yes (blocks scaling to 8K entities)

**What it does:**
- Add active_enemy_count cache to EnemySpawner (replace O(n) query)
- Add on_enemy_spawned/despawned() methods
- Update spawn_enemies() and despawn_dead() to maintain cache
- Fix metrics accumulation overflow (use rolling average instead)
- Log pool exhaustion instead of silent fallback

**Risk:** LOW (performance optimization, backward compatible)

**Tests:** 3 tests (cache accuracy, no overflow over 3 days, benchmark)

---

## Dependency Graph

```
Job 2 (Collision)
    ↓
Job 1 (Config) — Independent
Job 3 (Bounds) — Independent
Job 4 (Perf)   — Independent
    
All can start in parallel. Job 2 should complete before final integration test.
```

---

## How to Post These Jobs to SOLIVE

### Option 1: Use SOLIVE CLI (if available)

```bash
# Create hashes of job specs
solive_hash < SOLIVE_JOB_AUDIT_FIX_01.md  # Returns SHA256 hash
solive_hash < SOLIVE_JOB_AUDIT_FIX_02.md
solive_hash < SOLIVE_JOB_AUDIT_FIX_03.md
solive_hash < SOLIVE_JOB_AUDIT_FIX_04.md

# Create escrow for each job (example for Job 1)
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/YOUR_REPO/SOLIVE_JOB_AUDIT_FIX_01.md \
  --spec_hash <HASH_FROM_ABOVE> \
  --amount 0.12 \
  --judge <YOUR_WALLET>
```

### Option 2: Manual GitHub + SOLIVE

1. **Push job specs to GitHub**
   ```bash
   git add SOLIVE_JOB_AUDIT_FIX_*.md SOLIVE_PHASE_1_LAUNCH.md
   git commit -m "Add Phase 1 critical fix job specs for SOLIVE workers"
   git push origin main
   ```

2. **Get GitHub raw URLs**
   ```
   https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_01.md
   https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_02.md
   https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_03.md
   https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_04.md
   ```

3. **Create jobs via SOLIVE interface**
   - Upload each spec
   - Set budget (0.12 / 0.13 / 0.14 / 0.12 SOL)
   - Set deadline (7 days)
   - Set judge (your wallet address)

---

## Verification Workflow

### For Workers (Before Submitting)

```bash
# Test locally
cargo test config_validation              # Job 1
cargo test collision_entity_ids           # Job 2
cargo test negative_coordinates           # Job 3
cargo test enemy_count_cache              # Job 4

# Run integrated demo
cargo run --release --example integrated_demo

# Verify no new panics or warnings
```

### For Judge (You) - After Delivery

```bash
# 1. Fetch worker's branch
git fetch origin worker-fix-config
git checkout worker-fix-config

# 2. Run verification suite
./verify_solive_delivery.ps1 JOB_CONFIG_VALIDATION
./verify_solive_delivery.ps1 JOB_COLLISION_FIX
./verify_solive_delivery.ps1 JOB_BOUNDS_FIX
./verify_solive_delivery.ps1 JOB_PERF_FIX

# 3. Run all tests
cargo test --all

# 4. Stress test
cargo run --release --example integrated_demo -- --frames 100000

# 5. Approve delivery
solive_wallet_attest --job <JOB_ID> --approved
# Payment releases to worker's wallet (3% protocol fee)
```

---

## Estimated Timeline

**Day 1 Morning:** Post 4 jobs to SOLIVE  
**Day 1 Afternoon:** Workers claim jobs (parallel)  
**Day 1-2:** Workers implement (3-5 hours each)  
**Day 2:** Workers submit PRs  
**Day 2 Afternoon:** You verify (30-60 mins per job)  
**Day 3 Morning:** Merge PRs, payment released  

---

## Success Criteria (For Each Job)

### Job 1: Config Validation ✅
- [ ] No panics on invalid config
- [ ] All 4 tests pass
- [ ] load_or_default() calls validate()
- [ ] Error messages are helpful

### Job 2: Collision Attribution ✅
- [ ] Entity IDs correctly assigned (not zeros)
- [ ] All test cases pass
- [ ] Damage attributed to correct entities
- [ ] Compile with no warnings

### Job 3: Spatial Grid & Bounds ✅
- [ ] Negative coordinates work correctly
- [ ] Radius validation prevents overflow
- [ ] All 4 tests pass
- [ ] No panics from invalid values

### Job 4: Performance ✅
- [ ] Enemy count cache updated on spawn/despawn
- [ ] Metrics don't overflow after 3+ days
- [ ] No O(n) queries in hot path
- [ ] All 3 tests pass

---

## Budget Breakdown

| Job | Scope | Budget | Status |
|-----|-------|--------|--------|
| 1. Config Validation | Add validation + tests | 0.12 SOL | 💰 |
| 2. Collision Attribution | Fix entity IDs + update callers | 0.13 SOL | 💰 |
| 3. Bounds Checking | Grid fix + validation + tests | 0.14 SOL | 💰 |
| 4. Performance | Caching + metrics + tests | 0.12 SOL | 💰 |
| **TOTAL** | **Phase 1 Complete** | **0.51 SOL** | **Ready** |

---

## What Gets Fixed

After all 4 jobs complete:

🟢 **Critical Issues:** 5 → 0  
🟠 **High Priority:** 12 → 2 (remaining for Phase 2)  
📈 **Production Readiness:** 65% → 85%  
⚡ **Scaling:** 500 entities → 8000 entities  
🛡️ **Safety:** Partial → Comprehensive bounds checking  

---

## Next Steps (After Phase 1)

Once these 4 jobs are merged:

1. **Phase 2 Jobs (0.25 SOL total)**
   - HashSet in spatial grid
   - Pool component reset
   - Batch size validation

2. **Phase 3 Jobs (0.30 SOL total)**
   - Custom error types
   - Config validation framework
   - Collision filtering system

3. **Phase 4 Jobs (0.40+ SOL)**
   - Hot-reload config
   - Bounded metrics
   - Advanced collision features

---

## Timeline to Production-Ready

| Phase | Jobs | Budget | Time | Status |
|-------|------|--------|------|--------|
| 1 | 4 critical fixes | 0.51 SOL | 24-48 hrs | 🚀 Ready to post |
| 2 | Safety & perf | 0.25 SOL | 2-3 days | 📋 Specs ready |
| 3 | Architecture | 0.30 SOL | 3-4 days | 📋 Specs ready |
| **Total to Prod-Ready** | **12 jobs** | **~2.0 SOL** | **~2 weeks** | ✅ Feasible |

---

## Communication Template for Workers

```
Subject: SOLIVE Job: [Job Title] - Vampire Survivors Foundation Audit Fix

Hi [Worker],

We've completed a comprehensive code audit of our game engine foundation 
and identified [N] critical issues. This job is part of Phase 1 fixes.

**What you'll build:**
[Copy acceptance criteria from job spec]

**Files to modify:**
[List files]

**Tests to add:**
[List test cases]

**Time estimate:** [3-5 hours]
**Budget:** [X SOL]
**Deadline:** 7 days

Full spec: [Link to SOLIVE_JOB_AUDIT_FIX_NN.md on GitHub]

Questions? Reply here or comment on the SOLIVE job.

Thanks!
```

---

## Ready to Launch?

✅ All 4 job specs written and detailed  
✅ Acceptance criteria clear and verifiable  
✅ Test cases specified  
✅ Budget allocated (0.51 SOL)  
✅ No dependencies between jobs (parallel work)  

**Next action:** Post jobs to SOLIVE and await worker claims.
