# SOLIVE Phase 1 Deployment Checklist

**Status:** Ready to Deploy ✅  
**Date:** 2026-10-31  
**Total Budget:** 0.51 SOL (testnet)  

---

## Job Specs with Hashes

### Job 1: Config Validation System
- **File:** `SOLIVE_JOB_AUDIT_FIX_01.md`
- **Spec Hash:** `71c5c056c32217aae1a77432cb88894ec98c2bc562167966f53f412ad764b1e8`
- **Budget:** 0.12 SOL
- **GitHub URL:** `https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_01.md`
- **Description:** Add comprehensive validation for GameConfig to prevent invalid config from crashing game

### Job 2: Entity ID & Collision Attribution Fix
- **File:** `SOLIVE_JOB_AUDIT_FIX_02.md`
- **Spec Hash:** `25fe5b032702df0ce70d3e6fa943662fea77a2370f329af78479157eff5c3187`
- **Budget:** 0.13 SOL
- **GitHub URL:** `https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_02.md`
- **Description:** Fix critical bug where collision damage is attributed to entity 0 instead of actual entities

### Job 3: Spatial Grid & Bounds Checking
- **File:** `SOLIVE_JOB_AUDIT_FIX_03.md`
- **Spec Hash:** `024efa2bee3f7f5dc643d8b7f07ddd4b2ae250ab082d692bcebb72cc6efc1b65`
- **Budget:** 0.14 SOL
- **GitHub URL:** `https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_03.md`
- **Description:** Fix integer overflow with negative coordinates, add bounds checking

### Job 4: Performance Optimization & Metrics Fixes
- **File:** `SOLIVE_JOB_AUDIT_FIX_04.md`
- **Spec Hash:** `bce1b9f0346c788291749bf0df99414597f71e6908a076a0dafb3e093a3f5138`
- **Budget:** 0.12 SOL
- **GitHub URL:** `https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_04.md`
- **Description:** Remove O(n) queries, fix metrics overflow, cache entity counts

---

## Deployment Steps

### Step 1: Prepare GitHub URLs
Replace `YOUR_REPO` with your actual GitHub repository:
- Change `YOUR_REPO` → `MerlijnW70/game-editor` (example)

### Step 2: Get Your Wallet Address
You need your SOLIVE wallet address (attestor/judge):
```bash
solive_wallet_balance
# Returns: Your wallet: DxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxX
```

Copy your wallet address.

### Step 3: Create Job 1 Escrow (Preview)
```bash
solive_wallet_escrow \
  --sell SOL \
  --amount 0.12 \
  --attestor YOUR_WALLET_ADDRESS \
  --spec_url "https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_01.md" \
  --spec_hash "71c5c056c32217aae1a77432cb88894ec98c2bc562167966f53f412ad764b1e8" \
  --expires_in 604800
# Shows preview. Review it.
```

### Step 4: Send Job 1 Escrow (After Approval)
```bash
solive_wallet_escrow \
  --sell SOL \
  --amount 0.12 \
  --attestor YOUR_WALLET_ADDRESS \
  --spec_url "https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_01.md" \
  --spec_hash "71c5c056c32217aae1a77432cb88894ec98c2bc562167966f53f412ad764b1e8" \
  --expires_in 604800 \
  --approval "APPROVAL_STRING_FROM_PREVIEW" \
  --send
# Payment locked in escrow
```

### Step 5: Repeat for Jobs 2, 3, 4

**Job 2:**
```bash
solive_wallet_escrow \
  --sell SOL \
  --amount 0.13 \
  --attestor YOUR_WALLET_ADDRESS \
  --spec_url "https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_02.md" \
  --spec_hash "25fe5b032702df0ce70d3e6fa943662fea77a2370f329af78479157eff5c3187" \
  --expires_in 604800 \
  --send
```

**Job 3:**
```bash
solive_wallet_escrow \
  --sell SOL \
  --amount 0.14 \
  --attestor YOUR_WALLET_ADDRESS \
  --spec_url "https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_03.md" \
  --spec_hash "024efa2bee3f7f5dc643d8b7f07ddd4b2ae250ab082d692bcebb72cc6efc1b65" \
  --expires_in 604800 \
  --send
```

**Job 4:**
```bash
solive_wallet_escrow \
  --sell SOL \
  --amount 0.12 \
  --attestor YOUR_WALLET_ADDRESS \
  --spec_url "https://raw.githubusercontent.com/YOUR_REPO/main/SOLIVE_JOB_AUDIT_FIX_04.md" \
  --spec_hash "bce1b9f0346c788291749bf0df99414597f71e6908a076a0dafb3e093a3f5138" \
  --expires_in 604800 \
  --send
```

---

## Step 6: Verify Jobs Posted

```bash
solive_find_jobs --attestor YOUR_WALLET_ADDRESS --limit 10
# Should show 4 new jobs
```

---

## Step 7: Worker Claims & Delivery

Workers will:
1. See jobs via `solive_find_jobs`
2. Read full spec from GitHub URL
3. Verify spec_hash matches
4. Claim job (optional, if claim_stake is set)
5. Implement locally
6. Run tests
7. Deliver with `solive_wallet_deliver --url https://github.com/USERNAME/game-editor/pull/123`

---

## Step 8: Judge (Your) Verification

For each delivered job:

```bash
# 1. Fetch worker's code
git fetch origin worker-fix-config

# 2. Verify it works
cargo test --all
cargo run --release --example integrated_demo -- --frames 10000

# 3. Check specific tests
cargo test config_validation              # Job 1
cargo test collision_entity_ids           # Job 2
cargo test negative_coordinates           # Job 3
cargo test enemy_count_cache              # Job 4

# 4. Approve delivery
solive_wallet_attest --job JOB_ID --approved

# Payment releases automatically (97% to worker, 3% protocol fee)
```

---

## Budget Summary

| Job | File | Hash (first 16 chars) | Budget | Status |
|-----|------|----------------------|--------|--------|
| 1 | SOLIVE_JOB_AUDIT_FIX_01.md | 71c5c056c322... | 0.12 SOL | ✅ Ready |
| 2 | SOLIVE_JOB_AUDIT_FIX_02.md | 25fe5b032702... | 0.13 SOL | ✅ Ready |
| 3 | SOLIVE_JOB_AUDIT_FIX_03.md | 024efa2bee3f... | 0.14 SOL | ✅ Ready |
| 4 | SOLIVE_JOB_AUDIT_FIX_04.md | bce1b9f0346c... | 0.12 SOL | ✅ Ready |
| **TOTAL** | **4 Job Specs** | **Hashed & Ready** | **0.51 SOL** | **🚀 Launch Ready** |

---

## Quick Reference

**Your Wallet Address:** (Get from `solive_wallet_balance`)  
**Your GitHub Repo:** (e.g., MerlijnW70/game-editor)  
**Network:** Testnet (Devnet)  
**Deadline:** 7 days per job  
**Verification Command:** `solive_find_jobs --attestor YOUR_ADDRESS`

---

## What Happens After Posting

1. **Workers see jobs** (within minutes)
2. **Workers claim jobs** (can work in parallel on all 4)
3. **Workers deliver code** (24-48 hours typical)
4. **You verify** (30-60 min per job)
5. **Payments release** (automatic on approval)

---

## Support

Issues? Check:
- Job specs in `SOLIVE_JOB_AUDIT_FIX_*.md`
- Launch guide in `SOLIVE_PHASE_1_LAUNCH.md`
- Verification tests in each spec

**Timeline to Production:** Phase 1 (24-48 hrs) → Phase 2 (2-3 days) → Phase 3 (3-4 days) = ~2 weeks total to production-ready foundation.
