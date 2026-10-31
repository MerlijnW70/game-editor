# Codebase Optimization Plan

**Codebase Size:** 3,890 lines  
**Target:** Quick wins + reference implementations  
**Focus:** Quality baseline for Phase 1 workers

---

## Quick Win Optimizations

### 1. Collision Detection Optimization
**Location:** src/collision/detection.rs  
**Current:** O(n) entity iteration in hot path  
**Optimization:** Cache entity count, use early exit on collision found  
**Impact:** 10-20% faster collision detection  
**Time:** 1-2 hours  

### 2. Movement System Caching
**Location:** src/systems/movement.rs  
**Current:** Recalculates player direction every frame  
**Optimization:** Cache direction, only recalculate on input change  
**Impact:** 5-10% faster movement system  
**Time:** 30 minutes  

### 3. Profiler Metrics Optimization
**Location:** src/debug/profiler.rs  
**Current:** Allocates new metrics every frame  
**Optimization:** Pool metrics buffer, reuse allocations  
**Impact:** Eliminate allocation churn in profiler  
**Time:** 1 hour  

### 4. Despawn System Batch Processing
**Location:** src/systems/despawn.rs  
**Current:** Despawns one entity per iteration  
**Optimization:** Batch despawns, reduce command overhead  
**Impact:** 30-40% faster despawn for many dead entities  
**Time:** 1 hour  

### 5. Pool Manager Improvements
**Location:** src/pool.rs  
**Current:** HashMap lookups on acquire/release  
**Optimization:** Fast-path for common pool types  
**Impact:** 20-30% faster pool operations  
**Time:** 1-2 hours  

---

## Reference Implementation Priorities

### High Priority (Do First)
1. **Custom Error Types (Job #8)** - Foundation for error handling
2. **Config Validation Framework (Job #9)** - Validation patterns
3. **Collision Filtering (Job #10)** - Layer-based system

### Medium Priority (Do Next)
4. **Physics System (Job #16)** - Reference for force/dynamics
5. **Advanced Collision Shapes (Job #17)** - SAT algorithm example

### Low Priority (Can Wait)
6. **Query Pooling** - Example buffer management
7. **Cell Buffer Pre-allocation** - SIMD-friendly patterns

---

## Testing Framework Expansion

### Add Benchmarks
- Collision detection (circle, rectangle, polygon)
- Grid updates (1K, 8K, 16K entities)
- Despawn performance (10, 100, 1000 entities)
- Pool operations (acquire, release, reset)

### Expand Unit Tests
- Edge cases for each optimization
- Performance regression tests
- Memory usage tracking

### Stress Test Suite
- 8K entity + 1000 projectiles test
- Rapid spawn/despawn cycles
- Burst collision events

---

## Monitoring & Verification Tools

### Automated Test Runner
```bash
cargo test --release
cargo bench --all
cargo run --release --example integrated_demo -- --stress-test
```

### Performance Dashboard
Track:
- Frame time (avg, min, max)
- Collision checks per frame
- Memory allocations
- Entity counts by type

### Worker Verification Checklist
- [ ] All tests pass
- [ ] Performance benchmarks meet targets
- [ ] No new regressions
- [ ] Memory usage stable

---

## Implementation Order

**Week 1 of Phase 1 (While workers implement):**

Day 1-2:
- Collision detection optimization
- Movement caching
- Quick wins testing

Day 3-4:
- Profiler metrics optimization
- Despawn batching
- Pool manager fast-path

Day 5+:
- Phase 3 reference implementations
- Testing framework expansion
- Monitoring tools

---

## Success Metrics

| Optimization | Before | Target | Status |
|---|---|---|---|
| Collision/frame | 50 checks | 25 checks | |
| Movement update | 1.5ms | 1.2ms | |
| Despawn (100 ent) | 2.0ms | 1.2ms | |
| Pool ops | 500ns | 300ns | |
| Total frame time | 8-10ms | 6-8ms | |

