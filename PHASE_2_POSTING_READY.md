# Phase 2 Deployment: Ready for Posting

**Target Date:** November 1, 2026 (Day 3 PM - after Phase 1 approval)  
**Budget:** 0.19 SOL  
**Timeline:** 2-3 days for completion  
**Effort:** 3 jobs | 6-9 hours total work

---

## 📦 What Phase 2 Delivers

**High-Priority Safety & Performance Improvements**

After Phase 2, the foundation will:
- ✅ Use HashSet for O(1) duplicate detection in spatial grid
- ✅ Reset pool components safely on entity recycle
- ✅ Validate GPU batch sizes (prevent crashes)
- ✅ Reduce collision check overhead by 30-50%
- ✅ Eliminate duplicate entity processing

---

## 🎯 Phase 2 Job Specs

All 3 job specs are complete and ready to post:

### Job 5: HashSet Deduplication (0.06 SOL)

**File:** `SOLIVE_JOB_AUDIT_FIX_05.md`  
**Effort:** 2-3 hours  
**Focus:** Spatial grid collision deduplication

**What it fixes:**
- Currently: Same entity checked multiple times (O(n) duplicates)
- After: Unique entities only (O(1) dedup with HashSet)

**Quick Spec:**
```
- Add HashSet tracking in SpatialGrid
- query_circle() returns only unique entities
- 30-40% fewer collision checks
- Benchmark: <1ms at 8K entities
```

**Test command:**
```bash
cargo test spatial_grid_dedup
cargo test unique_entity_results
```

---

### Job 6: Pool Component Reset (0.06 SOL)

**File:** `SOLIVE_JOB_AUDIT_FIX_06.md`  
**Effort:** 2-3 hours  
**Focus:** Entity pool recycling safety

**What it fixes:**
- Currently: Recycled entities keep old components (memory leak)
- After: Components reset on entity recycle (clean reuse)

**Quick Spec:**
```
- Add reset() method to PooledEntity
- Clear Health, Position, Velocity on recycle
- Prevent stale component data
- Benchmark: <0.1ms reset overhead
```

**Test command:**
```bash
cargo test pool_reset_components
cargo test recycled_entity_clean
```

---

### Job 7: GPU Batch Size Validation (0.07 SOL)

**File:** `SOLIVE_JOB_AUDIT_FIX_07.md`  
**Effort:** 2-3 hours  
**Focus:** GPU rendering safety

**What it fixes:**
- Currently: Invalid batch sizes crash renderer
- After: Validation prevents invalid configs

**Quick Spec:**
```
- Add batch size validation (32-1024)
- Config loading checks limits
- Error handling on invalid size
- Benchmark: <0.5ms validation at startup
```

**Test command:**
```bash
cargo test batch_size_validation
cargo test gpu_batch_limits
```

---

## 📋 Posting Checklist

```bash
# Step 1: Check wallet balance (must cover the total the generator prints: 0.19 SOL)
solive_wallet_balance
```

Step 2: Generate the commands — never hand-write a hash or URL. The generator hashes the spec as
committed on `origin/main`, pins the URL to that commit, reads the amount from the spec, and
refuses jobs whose `Depends On` jobs are not approved yet:

```bash
# --approved: jobs already approved AND merged to main
bash scripts/solive_post_commands.sh \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm \
  --approved 1,2,3,4 \
  phase2
```

Run the printed `solive_wallet_escrow` commands in the session that has the SOLIVE tools.

```bash
# Step 3: Verify posting
solive_find_jobs --min_pay 0.05
# Should see Phase 2 jobs in results
```

---

## 📊 Phase 2 Job Summary

| Job | Title | Budget | Effort | Files Modified |
|-----|-------|--------|--------|-----------------|
| 5 | HashSet Dedup | 0.06 | 2-3h | src/collision/spatial.rs |
| 6 | Pool Reset | 0.06 | 2-3h | src/pool.rs |
| 7 | Batch Validation | 0.07 | 2-3h | src/rendering/gpu_batch.rs, src/resources/config.rs |

**Total Phase 2:** 0.19 SOL | 6-9 hours

---

## 🚀 Posting Workflow

**When Phase 1 Complete (Nov 1, 2:00 PM):**

1. Approve & pay all 4 Phase 1 jobs
2. Run this checklist
3. Post Phase 2 jobs (takes ~5 minutes)
4. Workers discover jobs immediately
5. Implementation starts (completes Nov 3)

**Timeline:**
```
Nov 1, 2:00 PM - Phase 1 approval complete
Nov 1, 2:30 PM - Phase 2 jobs posted
Nov 1, 2:45 PM - Workers claiming jobs
Nov 1-3 - Phase 2 implementation (parallel)
Nov 3, 3:00 PM - Phase 2 deliveries expected
```

---

## 💡 Why Post Phase 2 Immediately?

- ✅ Workers can start immediately (no idle time)
- ✅ Jobs 5-7 are independent (can work in parallel)
- ✅ Fixes are high-priority (safety + performance)
- ✅ Small budget (0.19 SOL) = quick ROI
- ✅ Fast wins build momentum

---

## 🔗 Reference Files

**Job Specs (Ready Now):**
- `SOLIVE_JOB_AUDIT_FIX_05.md` - HashSet dedup
- `SOLIVE_JOB_AUDIT_FIX_06.md` - Pool reset
- `SOLIVE_JOB_AUDIT_FIX_07.md` - Batch validation

**Timeline:**
- `PRODUCTION_ROADMAP.md` - Full 2-week timeline
- `DEPLOYMENT_COMPLETE.md` - Phase 1 deployment status
- `PHASE_1_VERIFICATION_GUIDE.md` - How to verify Phase 1

**Upcoming:**
- Phase 3 (Jobs 8-10): Architecture foundation
- Phase 4 (Jobs 11-15): Production features
- Phase 5 (Jobs 16-17): Physics system

---

## ✅ Pre-Posting Verification

- [ ] All 3 job specs exist and are complete
- [ ] Specs follow Phase 1 format/structure
- [ ] Test cases are comprehensive
- [ ] Acceptance criteria are clear
- [ ] Effort estimates are accurate
- [ ] Wallet has sufficient balance (>0.19 SOL)
- [ ] Spec fixes merged to `main` before generating commands

---

**Status:** ✅ Phase 2 Ready for Posting  
**Next Action:** Post on Nov 1 after Phase 1 approval

