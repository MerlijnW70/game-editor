# Vampire Survivors-Style Game Engine - Solive Research Tasks

This project uses **solive** to hire distributed workers to build the technical foundation for a world-class game engine inspired by Vampire Survivors.

## Project Structure

The project is broken into 4 parallel tasks, each with a detailed specification:

### Task 1: Core ECS + Entity Management System
**Spec Hash**: `776eff593c956dc6bacf90723e6e51234d3919889454a6bd3ee655c10293cb09`
- Build Entity Component System foundation in Bevy
- Implement core components (Transform, Velocity, Health, Collision, Sprite, Team)
- Payment: 0.05 SOL

### Task 2: 2D Rendering Pipeline with Batching & Camera
**Spec Hash**: `e7aa31b3cb9cfbd3c78b1b7b525fe7fc9bd62346c9e86147e1a8ac80c1a3f98a`
- Implement sprite batching and efficient rendering
- Camera system with smooth following
- Layer/Z-index sorting system
- Payment: 0.05 SOL

### Task 3: Collision Detection & Physics System
**Spec Hash**: `c110faa63c11a52510f7c5f346523032cff5aa474f3649c87fa83846e31e8405`
- Spatial partitioning (quadtree/grid)
- Efficient collision detection for 100+ entities
- Knockback and collision response
- Payment: 0.05 SOL

### Task 4: Integrated Demo Scene - Playable Prototype
**Spec Hash**: `6b52b95fc0e7c041f4a3ec851568319f0eadd863a00f510be5ff4e78cf82da25`
- Integrate all 3 systems into playable game
- Player controls, enemy AI, projectile system
- Proof of 60+ FPS with 100+ entities
- Payment: 0.05 SOL

## Total Budget
**0.20 SOL** for all 4 tasks (each 0.05 SOL)

## Judge
Wallet: `2atXgJgD7AdDiJK6rah7yq4HHJEQQxQCvcKcoxFWTyKm`

## How to Use

1. Each spec file in this repo describes exactly what workers need to deliver
2. Workers find the corresponding solive job and claim it
3. Workers build and deliver code/proof
4. Judge reviews and approves work
5. Worker gets paid via escrow

## Tech Stack
- **Language**: Rust
- **Engine**: Bevy (game engine framework)
- **Target**: 60+ FPS with 100+ simultaneous entities

## Next Steps
Push this repo to GitHub and post solive jobs with spec URLs.
