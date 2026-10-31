# Worker Communication Templates

**Purpose:** Clear communication for workers and judges  
**Format:** Ready-to-use templates for all phases

---

## Phase 1 Job Announcement Template

```
Subject: SOLIVE Job: [Job Title] - Vampire Survivors Foundation Fix

Hi [Worker],

We're transforming a Vampire Survivors game engine foundation from 65% to 95% production-ready through distributed development.

**This Job:** [Job Title]
- Budget: [X] SOL
- Time estimate: [Y] hours
- Difficulty: Intermediate
- Deadline: 7 days

**What You'll Build:**
[Copy acceptance criteria from job spec]

**Files to Modify:**
[List files]

**Tests to Add:**
[List test cases]

**Success Criteria:**
[List acceptance criteria]

**Full Specification:**
https://raw.githubusercontent.com/[REPO]/main/SOLIVE_JOB_AUDIT_FIX_NN.md

**Submit:** Create a PR with your implementation. Make sure:
1. All tests pass locally (`cargo test --all`)
2. No clippy warnings (`cargo clippy --all`)
3. Performance benchmarks meet targets
4. Code is formatted (`cargo fmt`)

Questions? Reply here or comment on the SOLIVE job.

Thanks for building with us! 🚀
```

---

## Delivery Checklist (For Workers)

```
BEFORE YOU SUBMIT:

□ All unit tests pass
  cargo test --all --release
  
□ No clippy warnings
  cargo clippy --all
  
□ Benchmarks meet targets
  cargo bench --all
  
□ Code formatted
  cargo fmt
  
□ Stress test passes
  cargo run --release --example integrated_demo -- --frames 10000
  
□ Memory stable (no leaks)
  [Monitor task manager or valgrind]
  
□ Documentation complete
  [Code comments, doc examples]
  
□ Examples compile
  [Test all examples that use your code]

Checklist complete? Create PR and link to SOLIVE job! ✅
```

---

## Judge Verification Report Template

```
JOB VERIFICATION REPORT
======================

Job ID: [ID]
Job Title: [Title]
Worker: [Name]
Date: [Date]
Status: ✅ APPROVED / ❌ NEEDS WORK

TEST RESULTS
============
Unit Tests: ✅ [X]/[Y] passed
Integration Tests: ✅ [X]/[Y] passed
Benchmarks: ✅ Met targets
Stress Test: ✅ 10,000 frames stable

PERFORMANCE
===========
Frame Time: [X]ms (target: [Y]ms)
Memory Usage: [X]MB (stable)
No Regressions: ✅ Yes

CODE QUALITY
============
Clippy Warnings: 0
Test Coverage: [X]%
Documentation: ✅ Complete

ADDITIONAL NOTES
================
[Any observations, improvements suggested]

VERDICT
=======
This job meets all acceptance criteria and is approved for payment.
```

---

## Weekly Status Report Template

```
WEEKLY DEVELOPMENT STATUS
==========================

Week: [W] / Phase: [P]
Jobs Posted: [N]
Jobs Claimed: [N]
Jobs Delivered: [N]
Jobs Approved: [N]

METRICS
=======
Production Readiness: [X]% → [Y]%
Entity Scaling: [X]K → [Y]K sustainable
Frame Time: [X]ms → [Y]ms
Test Coverage: [X]% → [Y]%

COMPLETED THIS WEEK
===================
□ Job #N: [Title] - MERGED ✅
□ Job #N: [Title] - MERGED ✅
□ Job #N: [Title] - APPROVED ✅

IN PROGRESS
===========
□ Job #N: [Title] - 60% complete
□ Job #N: [Title] - Implementation phase

NEXT WEEK
=========
□ Post Job #N: [Title]
□ Post Job #N: [Title]
□ Merge Phase [P] if complete
□ Evaluate Phase [P+1] path

BLOCKERS/ISSUES
===============
[List any issues]

NEXT STEPS
==========
1. [Action]
2. [Action]
3. [Action]
```

---

## Performance Regression Alert Template

```
⚠️ PERFORMANCE REGRESSION DETECTED

Job: [Title] (ID: [ID])
System: [Movement/Collision/Rendering/etc]
Metric: [Frame time / Allocations / Queries / etc]

BEFORE: [X]ms
AFTER:  [Y]ms
REGRESSION: [+Z]% (target was [±5]%)

INVESTIGATION NEEDED
====================
□ Check for O(n) algorithms in hot path
□ Review allocation patterns
□ Verify grid queries are optimized
□ Check for unbounded loops

Please investigate before we approve this job.
```

---

## Deployment Readiness Checklist

```
PHASE COMPLETION CHECKLIST
==========================

□ All jobs posted for phase
□ All jobs claimed by workers
□ All jobs implementation started
□ All jobs under review
□ All tests passing
□ No performance regressions
□ Stress test 60,000 frames stable
□ Zero critical issues remaining
□ Documentation complete
□ Ready to merge

STATUS: [RED/YELLOW/GREEN]
EXPECTED MERGE DATE: [Date]
```

---

## Communication Schedule

### Daily
- Check job status (1x per day)
- Monitor deliveries
- Answer worker questions

### Per Delivery (20 mins)
- Review PR code
- Run verification suite
- Approve or request changes

### Phase Completion (1 hour)
- Full integration test
- Performance validation
- Merge to main
- Update status report

---

## Success Communication Template

```
🎉 PHASE [N] COMPLETE!

We've successfully completed Phase [N] of the Vampire Survivors 
foundation transformation!

📊 RESULTS:
- Production Readiness: [X]% → [Y]%
- Entity Scaling: [X]K → [Y]K
- Frame Time: [X]ms → [Y]ms
- Jobs Merged: [N]
- Tests Passing: [N]

✅ ACHIEVEMENTS:
[Bullet list of improvements]

🚀 NEXT:
Phase [N+1] launches [Date]
- New jobs: [List]
- Budget: [X] SOL
- Timeline: [Y] days

THANKS to all [N] workers who made this possible! 🙏

Ready for the next phase? See you then! 👋
```

