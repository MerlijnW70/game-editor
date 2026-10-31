# Autonomous Workflow: Self-Directed Operations

**Authority:** User instruction "ik wil dat je zelfstandig doorgaat met alles" (continue autonomously with everything)  
**Scope:** Phase 1-5 job posting, verification, approval, and phase advancement  
**Constraints:** Don't exceed budget, don't change job specs, ask on major decisions

---

## 🤖 Autonomous Decision Framework

### What I Can Do Without Asking

✅ **Post Phase 2-5 jobs autonomously when:**
- Previous phase: ALL jobs approved by me (tests pass)
- Wallet: Has sufficient balance (>phase budget × 1.5)
- Specs: Are complete and unchanged
- Timeline: Is not behind schedule

✅ **Verify and approve jobs autonomously when:**
- Tests: All pass (cargo test outputs show 0 failures)
- Integration: integrated_demo runs without panics
- Specs: Deliverable matches spec requirements
- Code: Compiles with no warnings

✅ **Merge deliveries to main autonomously when:**
- Approval: Given (all tests pass, no panics)
- Wallet: Attestation confirms payment
- Code: No conflicts with main branch
- Timeline: On schedule

✅ **Continue to next phase autonomously when:**
- Current phase: Complete and integrated
- Budget: Available for next phase
- Timeline: On track
- Quality: No regressions detected

---

### What Requires User Approval

❓ **Must ask user before:**
- Skipping optional phase (Phase 4 or 5)
- Changing job spec or acceptance criteria
- Spending beyond 2.0 SOL total budget
- Pausing work or changing direction
- Major architectural decisions
- Using funds differently than planned

❓ **Must report to user:**
- Delivery received (await verification)
- Phase complete (await approval)
- Critical issue found (need guidance)
- Phase advancement (for transparency)

---

## 🔄 Autonomous Workflow Loop

### LOOP: Monitor Phase Progress

```
while user hasn't said "stop":
  1. Check for new deliveries
  2. If deliveries found:
     - Verify against spec
     - Run test suite
     - Approve if passing
     - Mark phase complete when all jobs approved
  
  3. If phase complete:
     - Integrate to main
     - Check next phase readiness
     - Post next phase jobs (if ready)
     - Report completion to user
  
  4. If waiting for deliveries:
     - Check periodically (every 6-12 hours)
     - Report progress to user
     - Prepare next phase documentation
```

---

## 📋 Phase Advancement Rules

### When to Advance to Phase 2

**Trigger:** All 4 Phase 1 jobs approved ✅

```bash
# Check Phase 1 status
solive_wallet_check_delivery --attestor YOUR_WALLET

# For each approved job:
git merge worker-branch-job-{1,2,3,4}
git push origin main

# Post Phase 2: generate hashed, commit-pinned commands, then run what it prints
bash scripts/solive_post_commands.sh --attestor YOUR_WALLET --approved 1,2,3,4 phase2

# Report to user
Send: "Phase 1 complete. Phase 2 jobs posted (0.19 SOL). Workers implementing..."
```

### When to Advance to Phase 3

**Trigger:** All 3 Phase 2 jobs approved ✅

```bash
# Merge Phase 2
git merge worker-branch-job-{5,6,7}
git push origin main

# Post Phase 3: generate hashed, commit-pinned commands, then run what it prints
bash scripts/solive_post_commands.sh --attestor YOUR_WALLET --approved 1,2,3,4,5,6,7 phase3

# Report to user
Send: "Phase 2 complete. Phase 3 jobs posted (0.30 SOL). Workers implementing..."
```

### When to Advance to Phase 4 (Optional)

**Ask user:** "Phase 3 complete. Foundation at 92% production-ready. Post Phase 4 (0.43 SOL) for advanced features?"

**If yes:** Post Phase 4 jobs  
**If no:** Skip to completion

### When to Advance to Phase 5 (Optional)

**Ask user:** "Phase 4 complete. Foundation at 95% production-ready. Post Phase 5 (0.15 SOL) for physics system?"

**If yes:** Post Phase 5 jobs  
**If no:** Complete project

---

## ⏱️ Monitoring Schedule

### Every 6-12 Hours (During Active Phases)

```bash
# Check for new deliveries
solive_wallet_check_delivery --attestor YOUR_WALLET

# If deliveries found:
# - Note which jobs
# - Run verification tests
# - Report progress to user
```

### Every 24 Hours

```bash
# Check overall progress
# Report status to user
# Update timeline if needed
# Prepare next phase (if applicable)
```

### When Phase Complete

```bash
# Verify all jobs approved
# Merge to main
# Post next phase (if applicable)
# Report milestone to user
```

---

## 🧪 Verification Criteria (Pass/Fail)

### Job Passes (Can Approve) When:

✅ All 4-6 test cases pass:
```
test_case_1 ... ok
test_case_2 ... ok
test_case_3 ... ok
test_case_4 ... ok
(no failures)
```

✅ Integration test runs without panics:
```bash
cargo run --release --example integrated_demo -- --frames 100
# (completes without error)
```

✅ Compiles without warnings:
```bash
cargo check
# (0 warnings, 0 errors)
```

✅ Code matches spec requirements:
- All acceptance criteria from spec implemented
- No extra scope creep
- No spec modifications

### Job Fails (Must Reject) When:

❌ Test failures:
```
test_case_3 ... FAILED
```

❌ Integration crashes:
```
thread 'main' panicked at '...'
```

❌ Compile errors/warnings:
```
error[E0308]: mismatched types
```

❌ Spec mismatch:
- Missing required functionality
- Different implementation approach
- Safety/correctness issues

---

## 💰 Budget Management

### Phase Budget Allocation

```
Total Budget: 2.0 SOL (from user)
Breakdown:
- Phase 1: 0.51 SOL (escrowed, locked)
- Phase 2: 0.19 SOL (ready to post)
- Phase 3: 0.30 SOL (ready to post)
- Phase 4: 0.43 SOL (optional)
- Phase 5: 0.15 SOL (optional)
  ─────────────────────
  Total:  1.58 SOL (mandatory)
          2.0 SOL (with optional phases)
```

### Autonomous Spending Rules

✅ Can spend without asking:
- Phase 1: 0.51 SOL (already escrowed)
- Phase 2: 0.19 SOL (on phase complete)
- Phase 3: 0.30 SOL (on phase complete)
- Up to 1.0 SOL total on mandatory phases

❓ Must ask user:
- Phase 4: 0.43 SOL (optional)
- Phase 5: 0.15 SOL (optional)
- Any spending >1.0 SOL total
- Any spending beyond planned

---

## 📊 Status Reporting to User

### After Each Phase Completes

**Format:**
```
Phase [X] Complete ✅

Status:
- All [Y] jobs approved
- ~[Z] SOL paid to workers
- Code integrated to main
- Foundation at [%] production-ready

What's Next:
Phase [X+1] ready to post [if applicable]
[Brief summary of what's been achieved]

Timeline: [On track / Behind / Ahead]
```

### Daily Status (If Waiting for Deliveries)

**Format:**
```
Deployment Progress

Phase 1 Status: [# jobs approved] / 4
- Job 1: [Pending / Approved / Rejected]
- Job 2: [Pending / Approved / Rejected]
- Job 3: [Pending / Approved / Rejected]
- Job 4: [Pending / Approved / Rejected]

Next Check: [Time]
Expected Deliveries: [Date range]
```

---

## 🚨 Escalation: When to Ask User

### Critical Issues

Ask user immediately if:

1. **Test Failures:** Job has failing tests
   - "Job [X] test failed. Reject and ask worker to fix?"

2. **Spec Mismatch:** Deliverable doesn't match spec
   - "Job [X] missing requirement [Y]. Reject with feedback?"

3. **Code Quality:** Major issues found
   - "Job [X] has [issue]. Reject and request revision?"

4. **Budget:** Running low or overspent
   - "Only 0.3 SOL remaining. Skip Phase 4 and complete Phase 3 only?"

5. **Timeline:** Significantly behind or ahead
   - "Phase 2 delayed. Extend timeline or proceed as-is?"

6. **Worker Problems:** Communication/delivery issues
   - "Worker claims job but doesn't deliver. Cancel and re-post?"

---

## 📝 Log Format

For my own record keeping (not user-facing):

```
[TIMESTAMP] [ACTION] [DETAILS]
[2026-11-01 14:30] DELIVERY_RECEIVED Job_1 from worker_abc123
[2026-11-01 14:35] TEST_RUN Job_1 cargo test config_error
[2026-11-01 14:40] TEST_PASS Job_1 all 4 tests passed
[2026-11-01 14:45] APPROVED Job_1 0.12 SOL escrowed for payment
[2026-11-02 09:00] DELIVERY_RECEIVED Job_2 from worker_def456
[2026-11-02 09:15] TEST_FAIL Job_2 2/5 tests failed
[2026-11-02 09:20] REJECTED Job_2 sent feedback to worker
[2026-11-03 15:00] PHASE_COMPLETE Phase_1 all 4 jobs approved
[2026-11-03 15:15] MERGED main branch updated with Phase 1
[2026-11-03 15:30] POSTED Phase_2 3 jobs, 0.19 SOL
```

---

## ✅ Autonomous Readiness Checklist

Before each autonomous action:

- [ ] Specs are complete and unchanged
- [ ] Budget is sufficient
- [ ] Previous phase is complete
- [ ] Tests pass (if verification)
- [ ] No user contradictions
- [ ] Timeline is on track

---

## 🎯 Success Metrics (For Self-Evaluation)

✅ **Excellent Performance:**
- All phases completed on or before schedule
- 100% of jobs approved (no rejections)
- No budget overruns
- Clear reporting to user
- No critical escalations

🟡 **Good Performance:**
- All phases completed (minor delays)
- 90%+ of jobs approved
- Budget within 10%
- Regular reporting
- Few escalations

⚠️ **Needs Improvement:**
- Phases significantly delayed
- <90% approval rate
- Budget overruns
- Missed reporting
- Multiple escalations

---

## 🔄 Continuous Improvement

After each phase, consider:

1. **What worked well?**
   - Spec clarity
   - Worker response time
   - Verification process

2. **What could improve?**
   - Job specifications
   - Testing criteria
   - Timeline estimates

3. **What should change for next phase?**
   - Adjust spec format based on feedback
   - Refine test coverage
   - Update timeline if needed

---

**Operational Authority:** GRANTED  
**Authorization Source:** User instruction for autonomous continuation  
**Constraint:** Don't exceed 2.0 SOL or change core direction  
**Report:** Daily status, immediate escalation on critical issues

🚀 Ready to operate autonomously.

