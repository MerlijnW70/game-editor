# Task 2: 2D Rendering Pipeline with Batching & Camera

## Objective
Implement an efficient 2D sprite rendering pipeline with layer sorting, sprite batching, and smooth camera following.

## Technical Requirements

### 1. Rendering Components
- **Sprite Component**: Texture handle, size, color tint, rotation
- **Layer/ZIndex**: Depth sorting for draw order (background, enemies, player, effects, UI)
- **Camera Component**: Position, zoom level, bounds checking
- **Animator Component**: Current frame, animation speed, frame data

### 2. Core Systems

#### Sprite Batching System
- Group sprites by texture to minimize draw calls
- Batch sprites with same material/shader together
- Achieve < 20 draw calls for 100 entities
- Support texture atlasing for common sprites

#### Camera System
- Smooth follow of player (with optional lerp easing)
- Camera bounds (don't show outside map edges)
- Zoom control (for focus on player or wide view)
- Parallax support structure (for later background layers)

#### Layer/Sorting System
- Define render layers (0-10): Background, Ground, Enemies, Player, Projectiles, Effects, UI
- Sort entities by layer then Z-index before rendering
- Verify correct draw order (player on top of enemies, effects on top of all)

#### Animation System
- Spritesheet animation with frame timing
- Support looping and one-shot animations
- Interpolate between frames smoothly

### 3. Performance Targets
- 100 sprites on screen: < 2ms render time
- < 20 draw calls total (heavy batching)
- 60+ FPS with full rendering pipeline
- Memory efficient sprite atlasing

### 4. Visual Features
- Tint support (color overlay for damage indicator, etc)
- Rotation support (enemies facing direction of travel)
- Scale/flip support (mirroring sprites without extra assets)

## Deliverable Format
- Integrate into existing Bevy ECS project (or extend Task 1)
- Rendering systems in `src/systems/rendering/`
- Camera system in `src/systems/camera/`
- Asset loading in `src/assets/`
- Example scene in `examples/rendering_demo.rs` showing:
  - 100 sprites animated and batched
  - Camera following player
  - Multiple layers visible
  - Performance metrics overlay

## Acceptance Criteria
1. Sprite batching reduces draw calls to < 20 for 100 entities
2. Camera smoothly follows player with configurable easing
3. Layer system correctly sorts 5+ different entity types
4. Animation plays smoothly without frame skipping
5. Can render at 60+ FPS with 100+ on-screen sprites
6. No visual glitches (sprites rendering in wrong order, clipping)
7. Code is modular (systems can be composed independently)

## Debugging/Proof
- Include debug visualization: show draw call count, FPS counter, layer boundaries
- Render with and without batching to show performance difference
- Capture 100 entity scene at 60 FPS as proof

## Payment & Timeline
- Payment: 0.05 SOL
- Estimated time: 6-10 hours
- Deliverable: Extended Bevy project with rendering working

## Judge Verification
1. Build and run `examples/rendering_demo.rs`
2. Verify 100 sprites render smoothly at 60 FPS
3. Watch camera follow player smoothly
4. Check draw call count in debug output (should be < 20)
5. Verify sprite layering (check that draw order is correct across Z-levels)
6. Examine code for batching implementation efficiency
