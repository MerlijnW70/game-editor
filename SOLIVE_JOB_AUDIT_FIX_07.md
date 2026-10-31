# SOLIVE Job: Batch Size Validation & Optimization

**Budget:** 0.07 SOL  
**Deadline:** 7 days  
**Effort:** 2-3 hours  
**Status:** Phase 2 - Ready after Phase 1 complete  
**Depends On:** Job #1 (config validation)

---

## Overview

GPU batching system has no validation on batch configuration. This allows invalid batch sizes that either waste memory (256 when 64 is optimal) or cause out-of-bounds writes (8 when minimum is 32).

---

## Problem

Current code in `src/rendering/gpu_batch.rs`:
```rust
pub struct GpuBatchConfig {
    pub max_sprites_per_batch: usize,  // ❌ No validation!
}

// Invalid config accepted:
let config = GpuBatchConfig {
    max_sprites_per_batch: 3,  // Too small - shader expects 32+
};
```

Symptoms:
- Invalid batch size → out-of-bounds GPU memory access → crash
- Batch size > optimal → wasted GPU VRAM (256 sprites when 64 is ideal)
- Batch size not power-of-2 → shader misalignment → visual glitches

---

## Requirements

1. **Add batch size validation** (src/rendering/gpu_batch.rs)
   ```rust
   impl GpuBatchConfig {
       const MIN_BATCH_SIZE: usize = 32;     // Shader minimum
       const MAX_BATCH_SIZE: usize = 1024;   // VRAM limit
       const OPTIMAL_BATCH_SIZE: usize = 256; // Best perf/memory tradeoff
       
       pub fn new(max_sprites_per_batch: usize) -> Result<Self, BatchConfigError> {
           if max_sprites_per_batch < Self::MIN_BATCH_SIZE {
               return Err(BatchConfigError::BatchSizeTooSmall {
                   provided: max_sprites_per_batch,
                   minimum: Self::MIN_BATCH_SIZE,
               });
           }
           
           if max_sprites_per_batch > Self::MAX_BATCH_SIZE {
               return Err(BatchConfigError::BatchSizeTooLarge {
                   provided: max_sprites_per_batch,
                   maximum: Self::MAX_BATCH_SIZE,
               });
           }
           
           // Warn if not optimal
           if max_sprites_per_batch != Self::OPTIMAL_BATCH_SIZE {
               warn!("Batch size {} is not optimal, consider {}", 
                   max_sprites_per_batch, Self::OPTIMAL_BATCH_SIZE);
           }
           
           Ok(Self {
               max_sprites_per_batch,
           })
       }
       
       pub fn validate(&self) -> Result<(), BatchConfigError> {
           if self.max_sprites_per_batch < Self::MIN_BATCH_SIZE {
               return Err(BatchConfigError::BatchSizeTooSmall { 
                   provided: self.max_sprites_per_batch,
                   minimum: Self::MIN_BATCH_SIZE,
               });
           }
           if self.max_sprites_per_batch > Self::MAX_BATCH_SIZE {
               return Err(BatchConfigError::BatchSizeTooLarge {
                   provided: self.max_sprites_per_batch,
                   maximum: Self::MAX_BATCH_SIZE,
               });
           }
           Ok(())
       }
   }
   ```

2. **Add BatchConfigError enum** (src/rendering/gpu_batch.rs)
   ```rust
   #[derive(Debug, Clone)]
   pub enum BatchConfigError {
       BatchSizeTooSmall { provided: usize, minimum: usize },
       BatchSizeTooLarge { provided: usize, maximum: usize },
   }
   
   impl std::fmt::Display for BatchConfigError {
       fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
           match self {
               BatchConfigError::BatchSizeTooSmall { provided, minimum } => {
                   write!(f, "Batch size {} is below minimum {}", provided, minimum)
               }
               BatchConfigError::BatchSizeTooLarge { provided, maximum } => {
                   write!(f, "Batch size {} exceeds maximum {}", provided, maximum)
               }
           }
       }
   }
   ```

3. **Update GameConfig to validate batching** (src/config.rs)
   ```rust
   pub fn validate(&self) -> Result<(), ConfigError> {
       // Existing validation...
       
       // NEW: Validate batch size
       GpuBatchConfig::new(self.gpu_batch_size)?;
       
       Ok(())
   }
   ```

4. **Add tests** (tests/gpu_batch.rs)
   ```rust
   #[test]
   fn test_batch_size_too_small() {
       let result = GpuBatchConfig::new(16);  // < 32 minimum
       assert!(result.is_err());
       match result.unwrap_err() {
           BatchConfigError::BatchSizeTooSmall { provided, minimum } => {
               assert_eq!(provided, 16);
               assert_eq!(minimum, 32);
           }
           _ => panic!("Wrong error type"),
       }
   }
   
   #[test]
   fn test_batch_size_too_large() {
       let result = GpuBatchConfig::new(2048);  // > 1024 maximum
       assert!(result.is_err());
       match result.unwrap_err() {
           BatchConfigError::BatchSizeTooLarge { provided, maximum } => {
               assert_eq!(provided, 2048);
               assert_eq!(maximum, 1024);
           }
           _ => panic!("Wrong error type"),
       }
   }
   
   #[test]
   fn test_batch_size_valid_range() {
       // All values 32..=1024 should be accepted
       for size in [32, 64, 128, 256, 512, 1024] {
           let config = GpuBatchConfig::new(size);
           assert!(config.is_ok(), "Size {} should be valid", size);
       }
   }
   
   #[test]
   fn test_optimal_batch_size() {
       let config = GpuBatchConfig::new(256).unwrap();
       assert_eq!(config.max_sprites_per_batch, 256);
   }
   ```

5. **Memory footprint analysis** (src/rendering/gpu_batch.rs)
   ```rust
   /// # Memory Usage by Batch Size
   ///
   /// Each SpriteInstance is 64 bytes:
   /// - position: 8 bytes (Vec2)
   /// - rotation: 4 bytes (f32)
   /// - scale: 8 bytes (Vec2)
   /// - color: 16 bytes (Vec4)
   /// - texture_index: 4 bytes (u32)
   /// - layer: 4 bytes (f32)
   /// - padding: 8 bytes
   ///
   /// Batch memory:
   /// - 32 sprites:  2 KB per batch
   /// - 64 sprites:  4 KB per batch
   /// - 256 sprites: 16 KB per batch (OPTIMAL - fits L1 cache)
   /// - 512 sprites: 32 KB per batch
   /// - 1024 sprites: 64 KB per batch (near L2 limit)
   ```

---

## Acceptance Criteria

✅ `GpuBatchConfig::new()` validates batch size  
✅ `BatchConfigError` enum with 2 variants  
✅ Minimum batch size 32 enforced  
✅ Maximum batch size 1024 enforced  
✅ All 4 test cases pass  
✅ Config loading rejects invalid batch sizes  
✅ Memory analysis documented  
✅ Compiles with no warnings  

---

## Verification Command

```bash
cargo test batch_size_too_small
cargo test batch_size_too_large
cargo test batch_size_valid_range
cargo test optimal_batch_size
cargo run --release --example integrated_demo
# Should start without panicking on batch config
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_BATCH_VALIDATION
```

Expected: ✅ Invalid batch sizes rejected, valid range enforced

