# Task: GPU Batch Rendering with Instancing

## Objective
Implement **GPU-driven sprite instancing** to render 200+ sprites in a single draw call using Bevy 0.16+ capabilities.

## Why This Matters
- Current: 2D rendering sprite batching framework exists, but GPU instancing not implemented
- Target: 200+ sprites → 1 draw call (vs 200 individual draws)
- Critical for visual performance; architectural change if added later

## Technical Requirements

### 1. Sprite Instancing System
- **GPU buffer**: Create instance data buffer (position, rotation, scale, color per sprite)
- **Material instancing**: Implement custom `Material2d` that reads from instance buffer
- **Batch preparation**: System that collects all sprites, groups by texture, writes to GPU buffer

### 2. Implementation via Bevy 0.16+ GPU-Driven Rendering
- Use `bevy_sprite` instancing support (new in 0.16)
- OR implement custom render graph if instancing not available in 0.16
- Fallback: Manually batch via custom render pass

### 3. Performance Targets
- **200 sprites**: Single draw call (measure with gizmos)
- **Frame time**: Rendering < 2ms (down from 20ms without batching)
- **FPS**: 60+ with 200+ sprites on-screen
- Proof: Benchmark render time with 50/100/200/300 sprites

### 4. Integration with Existing Rendering
- Keep `Sprite` component unchanged (no breaking API)
- Add internal `SpriteInstance` buffer management
- System runs in `CoreSet::Render` to populate instance buffer each frame

### 5. Example & Tests
- `examples/gpu_instancing_demo.rs`: Render 200+ moving sprites, show FPS/draw calls
- Visualization: Debug overlay showing draw call count
- Test: Verify all sprites render correctly (no missing/overlapping)

## Deliverable Format
- `src/systems/rendering/instancing.rs` → GPU instancing implementation
- Update `src/systems/rendering.rs` to use instancing
- Custom shader (WGSL) if needed: `assets/shaders/sprite_instance.wgsl`
- Example: `examples/gpu_instancing_demo.rs` with 200+ sprites
- Tests: `tests/gpu_rendering.rs` covering correctness + performance
- Documentation: `GPU_INSTANCING.md` explaining shader & buffer layout

## Acceptance Criteria
1. Code compiles with Bevy 0.16+ (no warnings)
2. All sprites render correctly (no clipping/glitches)
3. Draw call count < 10 for 200 sprites (target: 1-4 calls)
4. Frame time rendering < 2ms (measured via diagnostics)
5. 60+ FPS with 200+ sprites on-screen
6. Example demo shows FPS counter + draw call count
7. Tests pass (correctness + performance benchmarks)
8. Git commit with clear message

## Payment
**0.10 SOL** (most complex system)

## Timeline
- **Estimated**: 8-10 hours (shader work + integration)
- **Deadline**: 7 days from job posting

## Notes
- If Bevy 0.16 instancing unavailable, implement custom render graph
- Compatibility: Must work with existing `Layer` + `ZIndex` sorting
- Performance critical: This is the rendering bottleneck at scale
