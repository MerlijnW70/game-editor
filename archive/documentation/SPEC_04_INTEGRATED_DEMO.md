# Task 4: Integrated Demo Scene - Playable Vampire Survivors Prototype

## Objective
Integrate ECS, Rendering, and Collision systems into a playable demo scene proving the game engine foundation works at scale.

## Requirements

### 1. Game Scene Setup
- **Player Character**: Controlled by WASD or arrow keys
  - Visible sprite that rotates toward mouse/analog input
  - Health bar above character
  - Collision radius for enemy detection

- **Enemy AI**: 100+ simple enemies
  - Spawn in waves or continuously
  - Chase player using simple pathfinding (move toward player)
  - Deal damage on contact
  - Die when health reaches 0 with death animation

- **Projectile System**: Player fires projectiles
  - Keyboard input (spacebar or mouse click) spawns projectiles
  - Projectiles travel and despawn on enemy hit or timeout
  - Deal damage to enemies on contact
  - Visual feedback (muzzle flash, impact effect)

- **Game State**: 
  - Spawn enemies in increasing difficulty waves
  - Track player health and display UI
  - End game when player dies
  - Score: count of enemies defeated

### 2. Integration Requirements

#### All 3 Systems Working Together
- **ECS**: Player, enemies, projectiles managed as entities
- **Rendering**: 100+ sprites visible, animated, sorted by layer
- **Collision**: Enemy-player damage, projectile-enemy damage, no phasing

#### Input System
- WASD/Arrow keys for player movement
- Mouse or keyboard for aiming/shooting
- Smooth, responsive controls

#### Audio (Optional but Recommended)
- Enemy spawn sound
- Player damage sound
- Enemy death sound
- Background music

### 3. Performance Proof
- **Target**: 60+ FPS consistently with 100+ entities on screen
- Measure and log:
  - Frame time breakdown (ECS, Rendering, Collision)
  - Entity count over time
  - Memory usage
  - Draw call count

### 4. Visual Polish
- Clear visual difference between player, enemies, projectiles
- Health bars for player and enemies
- Death animations (fade out, knockback effect)
- Particle effects on impacts
- Score/wave counter UI

### 5. Playability
- Game feels responsive (no input lag)
- Difficulty ramps (more enemies each wave)
- Interesting gameplay (waves last 30-60 seconds before getting overwhelming)
- Clear win/loss conditions

## Deliverable Format
- Complete Bevy game project with all 3 systems integrated
- `src/main.rs` bootstraps the full game
- `src/game/` contains game logic (spawning, waves, AI)
- `src/systems/` contains all ECS systems from Tasks 1-3
- `assets/` contains sprites, sounds, fonts
- `examples/playable_demo.rs` or cargo run launches the game
- README with:
  - Controls
  - Performance metrics from benchmark run
  - Architecture overview
  - Known limitations
  - Next steps for expansion

## Acceptance Criteria
1. Game compiles and runs without crashes
2. Player can move smoothly around the map
3. 100+ enemies spawn and move toward player
4. Enemies deal damage to player (health decreases)
5. Player can shoot and kill enemies
6. Performance: Maintains 60+ FPS with 100+ entities
7. Game ends when player health reaches 0
8. Score tracks correctly (counts enemies killed)
9. Waves increase in difficulty over time
10. No rendering glitches, all entities visible and sorted correctly
11. No collision clipping or phasing
12. Game loop runs smoothly without stuttering

## Gameplay Video/Screenshot
Provide video evidence (or detailed screenshots) showing:
- Game running at 60 FPS
- 100+ enemies on screen
- Player dodging/fighting enemies
- Projectiles hitting enemies
- Score increasing
- Game difficulty ramping

## Performance Metrics Required
Run `examples/playable_demo.rs` for 2+ minutes and report:
- Average FPS (should be 60+)
- Max frame time (spike, should be < 20ms for 60 FPS)
- Breakdown: ECS time, Render time, Collision time, Other
- Peak entity count
- Memory usage at peak

## Payment & Timeline
- Payment: 0.05 SOL
- Estimated time: 10-16 hours (integration + gameplay)
- Deliverable: Playable game that proves the engine works

## Judge Verification Steps
1. `cargo build --release`
2. Run the game and play for 2+ minutes
3. Verify 100+ enemies are visible and moving
4. Test player controls (movement, shooting)
5. Watch enemies die and respawn
6. Check FPS counter (should be 60+)
7. View performance metrics in console output
8. Examine code architecture and system integration
9. Assess gameplay feel (responsiveness, fairness, fun factor)

## Success Definition
A playable Vampire Survivors-style game that:
- Runs at 60+ FPS with 100+ simultaneous entities
- Proves the ECS architecture scales
- Demonstrates rendering efficiency (sprite batching, sorting)
- Shows collision detection working at scale
- Is fun to play for 1-2 minutes
- Has clear next steps for full game development
