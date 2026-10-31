# Quick Reference: Commands & Operations

**Purpose:** Fast lookup for key operations during Phase 1-5 execution  
**Audience:** Claude autonomous system  
**Format:** Commands, outputs, and decision points

---

## 🔍 Check Deliveries

```bash
solive_wallet_check_delivery --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm
```

**What to look for:**
- Job ID (JOB_1, JOB_2, etc.)
- Worker address
- Submission timestamp
- Status: "pending_judgment"

**Action if found:** Run verification tests

---

## ✅ Verify Job Deliveries

### Job 1: Config Validation
```bash
git fetch origin <worker-branch>
git checkout <worker-branch>
cargo test config_error --lib
cargo test gameconfig_validate --lib
cargo run --release --example integrated_demo -- --stress-test --frames 100
```

**Pass criteria:** All tests pass, no panics

### Job 2: Collision Attribution
```bash
cargo test collision_entity_ids --lib
cargo test contactinfo_populated --lib
cargo run --release --example integrated_demo -- --collision-test --frames 100
```

**Pass criteria:** Entity IDs correct, no zeros

### Job 3: Bounds Checking
```bash
cargo test negative_coordinates --lib
cargo test spatial_bounds_safety --lib
cargo run --release --example integrated_demo -- --negative-coords --frames 500
```

**Pass criteria:** Negative coordinates work, no overflow

### Job 4: Performance Optimization
```bash
cargo test enemy_count_cache --lib
cargo test spawn_despawn_metrics --lib
cargo run --release --example integrated_demo -- --spawn-benchmark 8000
```

**Pass criteria:** <0.1ms per entity, cache working

---

## ✅ Approve Job

```bash
solive_wallet_attest --job JOB_ID --approved
```

**Requirement:** All verification tests passed

---

## ❌ Reject Job

```bash
solive_wallet_attest --job JOB_ID --rejected --feedback "Issue: [description]"
```

**Common rejections:**
- "Tests failing: test_case_3 FAILED"
- "Integration crash: panicked at '...'"
- "Missing spec requirement: [feature]"
- "Performance not improved: >1ms per entity"

---

## 📋 Merge Approved Job

```bash
# Checkout main
git checkout main
git pull origin main

# Merge job branch
git merge worker-branch-job-X --no-ff

# Push to main
git push origin main
```

---

## 💰 Check Wallet Balance

```bash
solive_wallet_balance
```

**Output:**
```
{
  "address": "2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm",
  "sol": "X.XXXXXXX",
  "tokens": [...]
}
```

**Key number:** `sol` field (must be >phase budget)

---

## 📤 Post Phase 2 Jobs

```bash
# Job 5: HashSet Dedup
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/.../SOLIVE_JOB_AUDIT_FIX_05.md \
  --spec_hash <HASH_HERE> \
  --amount 0.06 \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm

# Job 6: Pool Reset
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/.../SOLIVE_JOB_AUDIT_FIX_06.md \
  --spec_hash <HASH_HERE> \
  --amount 0.06 \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm

# Job 7: Batch Validation
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/.../SOLIVE_JOB_AUDIT_FIX_07.md \
  --spec_hash <HASH_HERE> \
  --amount 0.07 \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm
```

**Prerequisite:** Phase 1 complete (all 4 jobs approved)

---

## 📤 Post Phase 3 Jobs

```bash
# Job 8: Custom Error Types
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/.../SOLIVE_JOB_AUDIT_FIX_08.md \
  --spec_hash <HASH_HERE> \
  --amount 0.08 \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm

# Job 9: Config Framework
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/.../SOLIVE_JOB_AUDIT_FIX_09.md \
  --spec_hash <HASH_HERE> \
  --amount 0.10 \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm

# Job 10: Collision Filtering
solive_wallet_escrow \
  --spec_url https://raw.githubusercontent.com/.../SOLIVE_JOB_AUDIT_FIX_10.md \
  --spec_hash <HASH_HERE> \
  --amount 0.12 \
  --attestor 2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm
```

**Prerequisite:** Phase 2 complete (all 3 jobs approved)

---

## 🔗 Important URLs & Addresses

**Wallet:** `2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm`

**GitHub Repo:** (update with actual URL)
```
https://github.com/user/game-editor
```

**Spec Base URL:**
```
https://raw.githubusercontent.com/user/game-editor/main/
```

---

## 📊 Budget Checklist

```
Total: 2.0 SOL
├─ Phase 1: 0.51 SOL ✅ (escrowed)
├─ Phase 2: 0.19 SOL (ready)
├─ Phase 3: 0.30 SOL (ready)
├─ Phase 4: 0.43 SOL (optional)
└─ Phase 5: 0.15 SOL (optional)
```

**Decision Points:**
- After Phase 1: Have 0.51 SOL or more? ✓ Post Phase 2
- After Phase 2: Have 0.30 SOL or more? ✓ Post Phase 3
- After Phase 3: Ask user about Phase 4 (optional)
- After Phase 4: Ask user about Phase 5 (optional)

---

## 🎯 Decision Tree

```
┌─ Delivery Received
│  ├─ Run Tests
│  │  ├─ PASS → Approve (solive_wallet_attest --approved)
│  │  └─ FAIL → Reject (solive_wallet_attest --rejected --feedback "...")
│  │
│  └─ If APPROVED → Merge to main
│
├─ Phase Complete (All jobs approved)
│  ├─ Merge to main
│  ├─ Report to user
│  └─ Post Next Phase [if phase <= 3]
│
├─ Phase 3 Complete
│  └─ Ask User: Post Phase 4? [Y/N]
│
└─ Phase 4 Complete
   └─ Ask User: Post Phase 5? [Y/N]
```

---

## ⏱️ Monitoring Frequency

| Activity | Frequency | Command |
|----------|-----------|---------|
| Check deliveries | 6-12h | solive_wallet_check_delivery |
| Check wallet | Daily | solive_wallet_balance |
| Report status | Daily | Manual (via SendUserFile) |
| Post next phase | On trigger | solive_wallet_escrow |

---

## 📝 Example: Full Phase 1→2 Flow

```
1. [Nov 1] Phase 1 Jobs Posted ✅ (DONE)
   └─ 0.51 SOL escrowed

2. [Nov 2, AM] Delivery Received: Job 1
   └─ cargo test config_error → PASS
   └─ solive_wallet_attest --job JOB_1 --approved ✅

3. [Nov 2, PM] Delivery Received: Job 2
   └─ cargo test collision_entity_ids → PASS
   └─ solive_wallet_attest --job JOB_2 --approved ✅

4. [Nov 3, AM] All 4 Phase 1 Jobs Approved ✅
   └─ Merge all 4 branches to main
   └─ Report: "Phase 1 Complete"

5. [Nov 3, PM] Post Phase 2 Jobs
   └─ solive_wallet_escrow (Job 5, 6, 7)
   └─ 0.19 SOL escrowed

6. [Nov 3-4] Wait for Phase 2 Deliveries
   └─ Repeat process

7. [Nov 4, PM] Post Phase 3 Jobs
   └─ solive_wallet_escrow (Job 8, 9, 10)
   └─ 0.30 SOL escrowed

8. [Nov 5-6] Phase 3 Implementation

9. [Nov 7, AM] Phase 3 Complete ✅
   └─ Ask user: Post Phase 4?
```

---

## 🚨 Troubleshooting

| Problem | Solution |
|---------|----------|
| Delivery not showing | Wait 1 hour, check again |
| Test fails on one case | Reject with specific feedback |
| Wallet balance low | Check escrow status, wait for payment release |
| Worker doesn't respond | Job can be cancelled after timeout |
| Git merge conflict | Reject job, ask worker to rebase |
| Spec URL broken | Update URL in escrow command |

---

## ✅ Daily Checklist

```
[ ] Check solive_wallet_check_delivery for new deliveries
[ ] Run test suite on any deliveries found
[ ] Approve/reject jobs
[ ] Merge approved jobs to main
[ ] Check wallet balance (should be decreasing)
[ ] Post next phase when current phase complete
[ ] Report daily status to user
```

---

**Reference Card Status:** ✅ Ready for use  
**Last Updated:** 2026-10-31  
**Keep Handy:** Yes (used during all phases)

