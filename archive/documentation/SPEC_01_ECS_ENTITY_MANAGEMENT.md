# Task 1: Core ECS + Entity Management System

## Objective
Build the foundation Entity Component System (ECS) for a Vampire Survivors-style game engine in Bevy.

## Technical Requirements

### 1. Core Components (Must Implement)
- **Transform**: Position (Vec2), Rotation (f32), Scale (Vec2)
- **Velocity**: Movement vector (Vec2)
- **Health**: Current HP, Max HP, damage taken tracking
- **Damage**: Damage value, damage type (contact, projectile, ability)
- **Collision**: Collision radius, collision type (player, enemy, projectile, obstacle)
- **Sprite**: Asset path, size, animation state
- **Team**: Ownership (player, enemy, neutral) for collision filtering

### 2. Entity Archetypes
Implement factory functions for:
- **Player Archetype**: Transform + Velocity + Health(100) + Sprite + Input + Team(Player)
- **Enemy Archetype**: Transform + Velocity + Health(variable) + Sprite + AI + Team(Enemy)
- **Projectile Archetype**: Transform + Velocity + Damage + Sprite + Lifetime + Team

### 3. Systems to Implement
- **Movement System**: Update position based on velocity each frame
- **Despawn System**: Remove dead entities (health <= 0) and expired projectiles
- **Spawning System**: Efficient entity creation with pooling consideration
- **Health System**: Track damage events, handle death state

### 4. Performance Requirements
- Must handle creation/destruction of 500+ entities without memory fragmentation
- ECS query performance: < 1ms per frame for 100 concurrent entities
- Zero-copy component access where possible
- Demonstrate with `bevy_diagnostic` benchmarking

### 5. Testing & Verification
Create example scene showing:
- 50 enemies spawned and moving simultaneously
- Collision detection registering (no actual physics yet, just detection)
- Health system working (damage entities and verify health decreases)
- Entities despawning when health reaches 0
- Measure and report frame time

## Deliverable Format
- Bevy project (Cargo.toml, src/main.rs)
- Core systems in `src/systems/` directory
- Component definitions in `src/components/` directory
- Example scene in `examples/ecs_demo.rs`
- README with build/run instructions
- Performance benchmark results (FPS at 50 entities, 100 entities, 200 entities)

## Acceptance Criteria
1. Code compiles without warnings
2. Example runs at 60+ FPS with 50+ entities
3. All 5 components defined and used
4. All 4 systems implemented and working
5. Entity archetype factories demonstrate creating each type
6. Health/damage system verified working
7. No unsafe code (unless absolutely justified with comments)
8. Git repository with clear commit history

## Deadline & Payment
- Payment: 0.05 SOL
- Estimated time: 4-8 hours
- Deliverable: GitHub repo URL with working code

## Judge Verification Steps
1. Clone repo and run `cargo build`
2. Run `cargo run --example ecs_demo`
3. Verify 60+ FPS with performance output
4. Inspect code for architecture cleanliness
5. Run any included tests with `cargo test`
