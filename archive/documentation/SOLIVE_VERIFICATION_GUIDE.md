# Solive Delivery Verification Guide

## Overview

When workers deliver their implementations, you (as the judge/attestor) must:
1. Review the code
2. Run the spec's verification checks
3. Approve (pay) or reject (request revision)

## Quick Start

**Windows:**
```powershell
.\verify_solive_delivery.ps1
```

**Linux/Mac:**
```bash
bash verify_solive_delivery.sh
```

Choose a job name, the script will:
- Check if work has been delivered
- Fetch the deliverable
- Show you what to verify
- Provide approve/reject commands

---

## The 6 Jobs: What to Verify

### 1. SPATIAL_INDEXING (0.08 SOL)
**Deliverable:** Quadtree spatial partitioning implementation

**Verify:**
- [ ] Code compiles in Bevy 0.16+
- [ ] Collision queries run in O(log n) time
- [ ] Query speed measured < 1ms per frame for 100 entities
- [ ] All previous tests still pass
- [ ] Example: `examples/quadtree_demo.rs` shows query times

**Run:**
```bash
solive judge SOLIVE_SPEC_01_SPATIAL_INDEXING.md
```

---

### 2. MEMORY_POOLING (0.07 SOL)
**Deliverable:** Object pool system for entity recycling

**Verify:**
- [ ] PoolManager acquires entities < 1µs
- [ ] 1000 projectiles/frame spawn without allocation stalls
- [ ] No frame drops during high churn
- [ ] Pool demo benchmarks show gains over spawn_batch
- [ ] Integrated demo runs longer without entity overflow

**Run:**
```bash
solive judge SOLIVE_SPEC_02_MEMORY_POOLING.md
```

---

### 3. GPU_BATCHING (0.10 SOL)
**Deliverable:** GPU sprite instancing for 200+ sprites

**Verify:**
- [ ] Compiles with Bevy 0.16+ rendering
- [ ] 200 sprites render in < 10 draw calls (target: 1-4)
- [ ] Frame time < 2ms for sprite rendering
- [ ] 60+ FPS with 200+ on-screen entities
- [ ] Demo shows FPS counter + draw call count
- [ ] No visual glitches or missing sprites

**Run:**
```bash
solive judge SOLIVE_SPEC_03_GPU_BATCHING.md
```

---

### 4. CONFIG_SYSTEM (0.08 SOL)
**Deliverable:** YAML-driven configuration system

**Verify:**
- [ ] `assets/config/*.yaml` files load without errors
- [ ] Enemy stats read from config (hardcoded values removed)
- [ ] Item balance data loaded from config
- [ ] Wave progression customizable via config
- [ ] Config load time < 100ms
- [ ] Hot-reload works in dev builds
- [ ] Graceful fallback if config missing

**Run:**
```bash
solive judge SOLIVE_SPEC_04_CONFIG_SYSTEM.md
```

---

### 5. PREFAB_SYSTEM (0.09 SOL)
**Deliverable:** Template-based entity spawning from RON files

**Verify:**
- [ ] `assets/prefabs/*.ron` files parse correctly
- [ ] Entities spawn from templates with correct components
- [ ] Spawned entities collide/render/behave correctly
- [ ] Hot-reload works (file change → entity respawn)
- [ ] All component types deserialize
- [ ] Required components validated (no missing fields)

**Run:**
```bash
solive judge SOLIVE_SPEC_05_PREFAB_SYSTEM.md
```

---

### 6. DEBUG_PROFILER (0.10 SOL)
**Deliverable:** In-game performance profiler + entity inspector

**Verify:**
- [ ] Debug overlay shows FPS, frame time, system breakdown
- [ ] Press 'D' to toggle debug view on/off
- [ ] Entity inspector lists all entities + components
- [ ] Live-edit component values (health, position, speed)
- [ ] Pause (Spacebar), slow-mo (1-9 keys), step frame (F)
- [ ] Profiler overhead < 2% of frame time
- [ ] Spatial visualization shows collision shapes

**Run:**
```bash
solive judge SOLIVE_SPEC_06_DEBUG_PROFILER.md
```

---

## Approval Flow

### ✅ If All Tests Pass

1. Review the code in the work directory
2. Run the judge command (see above)
3. Confirm all checks pass
4. **Approve delivery:**

```bash
solive wallet attest \
  --intent <INTENT_ID> \
  --verdict approve \
  --feedback "Verified: all spec tests pass, code quality excellent"
```

**Result:** 
- 🎉 Payment releases from escrow to worker
- Worker wallet receives ~0.077 SOL (after 3% protocol fee)
- Code is accepted into the repository

### ❌ If Tests Fail

1. Note which checks failed
2. Document the specific issues
3. **Request revision:**

```bash
solive wallet refute \
  --intent <INTENT_ID> \
  --feedback "Fix: GPU batching only achieves 25 draw calls, target is <10. See profile_results.txt for details"
```

**Result:**
- 💼 Work stays in escrow
- Worker receives feedback
- Worker can resubmit revised work
- You re-verify and approve when ready

---

## Common Scenarios

### "Code compiles but tests fail"
→ Refute with specific feedback: "Compile OK, but memory_pooling_benchmark shows allocations during spawn. Target is zero allocations after pool init."

### "Some tests pass, some fail"
→ Refute. Worker can fix and resubmit. Payment stays locked until all tests pass.

### "Code works but doesn't follow spec format"
→ Refute. "Functionality works, but Config file location should be `assets/config/` per spec section 2.1, currently at `data/config/`"

### "Everything checks out"
→ Approve! Worker gets paid, code merges into main branch.

---

## Judge Commands Cheat Sheet

**Check delivery status:**
```bash
solive wallet check-delivery --intent <INTENT_ID>
```

**Fetch the work:**
```bash
solive wallet fetch-work --intent <INTENT_ID> --output ./work_review
```

**Run spec verification:**
```bash
solive judge SOLIVE_SPEC_XX_<NAME>.md
```

**Approve (release payment):**
```bash
solive wallet attest --intent <INTENT_ID> --verdict approve --feedback "..."
```

**Reject (request revision):**
```bash
solive wallet refute --intent <INTENT_ID> --feedback "..."
```

**Check payment status:**
```bash
solive wallet receipt --intent <INTENT_ID>
```

---

## Timeline

- **Now**: Jobs posted, 6 solive workers can claim
- **Days 1-3**: Workers implement in parallel
- **Days 2-6**: Workers submit deliveries as ready
- **Days 3-7**: You verify and approve/reject each job
- **Day 7**: Deadline expires, unclaimed/unpaid jobs revert

You're in full control. No payment leaves escrow until you sign off. ✨
