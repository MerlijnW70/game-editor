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
    pub max_sprites_per_batch: usize,  // ❌ No validation
    pub batch_size: usize,             // ❌ No validation — used as a divisor
    pub enabled: bool,
    pub draw_calls_last_frame: usize,
}

impl GpuBatchConfig {
    pub fn new(max_sprites_per_batch: usize) -> Self { /* accepts any value */ }

    pub fn batches_needed(&self, sprite_count: usize) -> usize {
        (sprite_count + self.batch_size - 1) / self.batch_size  // panics when batch_size == 0
    }
}
```

`GameConfig.rendering.sprite_batch_size` (`src/resources/config.rs`, loaded from
`assets/config/game.yaml`) is likewise accepted without any range check.

Symptoms:
- `batch_size == 0` → `batches_needed()` divides by zero → panic (also reached through
  `BatchingStats::calculate()` and `estimated_draw_calls()`)
- Absurd sizes (e.g. 3 or 100 000) are accepted silently from YAML

---

## Requirements

The engine's policy limits for a sprite batch are **32 ≤ size ≤ 1024**, with **256** as the
recommended default (the current `Default`). These are engine policy constants, not hardware
facts — keep them as named constants so they can be tuned in one place.

1. **Add a single range check and use it everywhere** (src/rendering/gpu_batch.rs)
   ```rust
   impl GpuBatchConfig {
       pub const MIN_BATCH_SIZE: usize = 32;
       pub const MAX_BATCH_SIZE: usize = 1024;
       pub const RECOMMENDED_BATCH_SIZE: usize = 256;

       /// Pure range check shared by `new()`, `validate()` and `GameConfig::validate()`.
       /// Emits no log output, so it is safe to call during config validation.
       pub fn check_size(size: usize) -> Result<(), BatchConfigError> {
           if size < Self::MIN_BATCH_SIZE {
               return Err(BatchConfigError::BatchSizeTooSmall { provided: size, minimum: Self::MIN_BATCH_SIZE });
           }
           if size > Self::MAX_BATCH_SIZE {
               return Err(BatchConfigError::BatchSizeTooLarge { provided: size, maximum: Self::MAX_BATCH_SIZE });
           }
           Ok(())
       }

       pub fn new(max_sprites_per_batch: usize) -> Result<Self, BatchConfigError> {
           Self::check_size(max_sprites_per_batch)?;
           if max_sprites_per_batch != Self::RECOMMENDED_BATCH_SIZE {
               warn!(
                   "Batch size {} differs from the recommended {}",
                   max_sprites_per_batch,
                   Self::RECOMMENDED_BATCH_SIZE
               );
           }
           Ok(Self {
               max_sprites_per_batch,
               batch_size: max_sprites_per_batch,
               ..Self::default()
           })
       }

       /// Re-checks a config whose public fields may have been mutated after construction.
       pub fn validate(&self) -> Result<(), BatchConfigError> {
           Self::check_size(self.max_sprites_per_batch)?;
           Self::check_size(self.batch_size)?;
           if self.batch_size > self.max_sprites_per_batch {
               return Err(BatchConfigError::BatchSizeExceedsMax {
                   batch_size: self.batch_size,
                   max_sprites_per_batch: self.max_sprites_per_batch,
               });
           }
           Ok(())
       }
   }
   ```

2. **Add the `BatchConfigError` enum** (src/rendering/gpu_batch.rs) and export it next to
   `GpuBatchConfig` in **both** `src/rendering/mod.rs` and `src/lib.rs`, so `tests/gpu_batch.rs`
   can import it as `survivors_engine::BatchConfigError`.
   ```rust
   #[derive(Debug, Clone, PartialEq, Eq)]
   pub enum BatchConfigError {
       BatchSizeTooSmall { provided: usize, minimum: usize },
       BatchSizeTooLarge { provided: usize, maximum: usize },
       BatchSizeExceedsMax { batch_size: usize, max_sprites_per_batch: usize },
   }

   impl std::fmt::Display for BatchConfigError {
       fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
           match self {
               BatchConfigError::BatchSizeTooSmall { provided, minimum } =>
                   write!(f, "Batch size {} is below minimum {}", provided, minimum),
               BatchConfigError::BatchSizeTooLarge { provided, maximum } =>
                   write!(f, "Batch size {} exceeds maximum {}", provided, maximum),
               BatchConfigError::BatchSizeExceedsMax { batch_size, max_sprites_per_batch } =>
                   write!(f, "batch_size {} exceeds max_sprites_per_batch {}", batch_size, max_sprites_per_batch),
           }
       }
   }

   impl std::error::Error for BatchConfigError {}
   ```

3. **Update existing callers of `GpuBatchConfig::new()`.** `new()` now returns a `Result`. The
   current callers are the unit tests in `src/rendering/gpu_batch.rs` (`test_batches_needed`,
   `test_estimated_draw_calls`, `test_batching_stats`, `test_memory_estimation`); change them to
   `GpuBatchConfig::new(256).expect("256 is within the batch size limits")`. Run
   `grep -rn "GpuBatchConfig::new" src examples tests` and update any other caller you find —
   do not leave a call site that ignores the `Result`.

4. **Validate `sprite_batch_size` in `GameConfig::validate()`** (src/resources/config.rs)

   Job #1 introduces `GameConfig::validate() -> Result<(), ConfigError>`, which collects every
   failure (returned as `ConfigError::MultipleFailed` when there is more than one) and already
   reports `ConfigError::BatchSizeZero`. Extend it — do not replace it:
   ```rust
   // New ConfigError variant (keep all of Job #1's variants)
   InvalidBatch(BatchConfigError),

   impl From<BatchConfigError> for ConfigError {
       fn from(e: BatchConfigError) -> Self { ConfigError::InvalidBatch(e) }
   }
   ```
   Inside `validate()`, follow Job #1's error-collection pattern (push into the same list) rather
   than returning early with `?`, so a config with several faults still reports all of them:
   - `sprite_batch_size == 0` → keep reporting **only** `BatchSizeZero` (one fault, one error)
   - otherwise, if `GpuBatchConfig::check_size(self.rendering.sprite_batch_size)` fails → record
     `ConfigError::InvalidBatch(e)`

   Extend `ConfigError`'s `Display` impl for the new variant.

5. **Add tests** (tests/gpu_batch.rs)
   ```rust
   #[test]
   fn batch_size_too_small() {
       assert_eq!(
           GpuBatchConfig::new(31).unwrap_err(),
           BatchConfigError::BatchSizeTooSmall { provided: 31, minimum: 32 }
       );
   }

   #[test]
   fn batch_size_too_large() {
       assert_eq!(
           GpuBatchConfig::new(1025).unwrap_err(),
           BatchConfigError::BatchSizeTooLarge { provided: 1025, maximum: 1024 }
       );
   }

   #[test]
   fn batch_size_valid_range() {
       // Both boundaries are inclusive.
       for size in [32, 64, 128, 256, 512, 1024] {
           let config = GpuBatchConfig::new(size).expect("size is within limits");
           assert_eq!(config.max_sprites_per_batch, size);
           assert_eq!(config.batch_size, size);
       }
   }

   #[test]
   fn batch_validate_rejects_mutated_fields() {
       let mut config = GpuBatchConfig::new(256).expect("256 is within limits");
       config.batch_size = 0;
       assert!(matches!(config.validate(), Err(BatchConfigError::BatchSizeTooSmall { provided: 0, .. })));

       let mut config = GpuBatchConfig::new(128).expect("128 is within limits");
       config.batch_size = 256;
       assert_eq!(
           config.validate(),
           Err(BatchConfigError::BatchSizeExceedsMax { batch_size: 256, max_sprites_per_batch: 128 })
       );
   }

   #[test]
   fn batch_config_rejects_out_of_range_sprite_batch_size() {
       let mut config = GameConfig::default();
       config.rendering.sprite_batch_size = 2048;
       // Assert the error contains ConfigError::InvalidBatch(BatchSizeTooLarge { provided: 2048, .. }),
       // either directly or inside MultipleFailed.
   }

   #[test]
   fn batch_config_zero_reports_only_batch_size_zero() {
       let mut config = GameConfig::default();
       config.rendering.sprite_batch_size = 0;
       // Assert exactly one batch-related error is reported, and it is ConfigError::BatchSizeZero.
   }

   #[test]
   fn batch_sprite_instance_layout() {
       // Guards the memory figures documented in step 6.
       assert_eq!(std::mem::size_of::<SpriteInstance>(), 48);
   }
   ```
   Fill in the two `GameConfig` tests against the `ConfigError` shape Job #1 delivered.

6. **Document memory footprint** (doc comment on `GpuBatchConfig` in src/rendering/gpu_batch.rs)
   ```rust
   /// # Memory usage by batch size
   ///
   /// `SpriteInstance` is 48 bytes (`#[repr(C)]`, enforced by `batch_sprite_instance_layout`):
   /// - position: 8 bytes ([f32; 2])
   /// - scale:    8 bytes ([f32; 2])
   /// - rotation: 4 bytes (f32)
   /// - layer:    4 bytes (f32)
   /// - color:   16 bytes ([f32; 4])
   /// - _padding: 8 bytes ([f32; 2])
   ///
   /// Instance data per full batch:
   /// - 32 sprites:   1.5 KiB
   /// - 256 sprites:  12 KiB (recommended default)
   /// - 1024 sprites: 48 KiB (maximum)
   ```
   If you change `SpriteInstance`, update this table and the layout test together.

---

## Acceptance Criteria

✅ `GpuBatchConfig::check_size()` is the only place the 32..=1024 range is encoded  
✅ `GpuBatchConfig::new()` returns `Result<Self, BatchConfigError>` and initialises every field  
✅ `GpuBatchConfig::validate()` checks `max_sprites_per_batch`, `batch_size`, and their ordering  
✅ `BatchConfigError` (3 variants, `Display` + `Error`) exported from `src/rendering/mod.rs` and `src/lib.rs`  
✅ Every caller of `GpuBatchConfig::new()` handles the `Result`  
✅ `GameConfig::validate()` reports out-of-range `sprite_batch_size` as `ConfigError::InvalidBatch`, and `0` only as `BatchSizeZero`  
✅ All 7 tests in `tests/gpu_batch.rs` pass, and the existing unit tests in `src/rendering/gpu_batch.rs` still pass  
✅ Memory table documented and guarded by the layout test  
✅ `cargo clippy --all-targets -- -D warnings` is clean  

---

## Verification Command

```bash
cargo test --test gpu_batch
cargo test --lib rendering::gpu_batch
cargo clippy --all-targets -- -D warnings
cargo run --release --example integrated_demo
# Should start without panicking on batch config
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_BATCH_VALIDATION
```

Expected: ✅ Invalid batch sizes rejected, valid range enforced

