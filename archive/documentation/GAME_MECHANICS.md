# Game Mechanics & Progression System

## Overview

This document describes the game mechanics, progression system, and difficulty scaling for the Vampire Survivors-inspired game engine.

---

## Core Mechanics

### Wave System

**Wave Progression:**
- Game starts at Wave 1
- Each wave lasts 30 seconds (decreases with difficulty)
- When wave ends, difficulty increases and next wave begins
- Boss wave every 5th wave (Wave 5, 10, 15, etc.)

**Difficulty Scaling:**
```
Wave 1:  30s, 20 enemies, health 30, speed 60
Wave 2:  27s, 30 enemies, health 35, speed 70
Wave 3:  25s, 40 enemies, health 40, speed 80
...
Wave 10: 15s (minimum), 120 enemies, health 80, speed 150 (BOSS!)
```

**Formula:**
- `Enemy Count = (20 + wave * 10).min(200) * difficulty_multiplier`
- `Max Health = 30 + (wave - 1) * 5`
- `Chase Speed = 60 + (wave - 1) * 10`
- `Difficulty Multiplier = 1.0 + (wave * 0.1)`
- `Wave Duration = (30 / difficulty_multiplier).max(15 seconds)`

---

## Scoring System

### Points & Kill Streaks

**Enemy Kill Value:**
- Each enemy killed = +1 point
- Points adjusted by kill streak multiplier

**Kill Streak Mechanic:**
- Counter increments with each enemy killed
- Counter resets to 0 when player takes damage
- Multiplier ranges from 1x to 5x based on streak:
  - 0-50 kills: 1x to 2x multiplier
  - 50-100 kills: 2x to 3x multiplier
  - 100-150 kills: 3x to 4x multiplier
  - 150+ kills: 4x to 5x multiplier

**Formula:**
```
streak_multiplier = 1.0 + (current_kills / 50.0).min(4.0)
```

### Score Tracking

**Game State Tracks:**
- `score`: Total enemy kills (current game)
- `current_kill_streak`: Current combo counter
- `max_consecutive_kills`: Best streak this game
- `time_survived`: Total time alive (seconds)

---

## Difficulty & Balance

### Automatic Difficulty Scaling

**Per-Wave Adjustments:**
1. Enemy count increases linearly
2. Enemy health increases with wave number
3. Enemy speed increases with wave number
4. Wave duration decreases (faster pacing)
5. Every 5th wave: Boss variant (40% more health, 30% faster)

### Player-vs-Enemy Balance

**Player Advantages:**
- Projectile attacks (velocity-based, infinite range)
- Health regeneration (via items, future)
- Temporary power-ups (future)
- Speed boost items (future)

**Enemy Advantages:**
- Outnumber the player (20+ vs 1)
- Herd behavior (swarm tactics)
- Sustained health (not fragile)

### Game-Over Conditions

1. **Player Health ≤ 0** → "Player defeated!"
2. **Max Entities Reached** → Game caps at 8192 entities (future: pooling fixes this)

---

## Progression Metrics

### Displayed to Player

- **Health**: Current player HP
- **Wave**: Current wave number
- **Score**: Total enemies killed
- **Wave Progress**: 0-100% through current wave (time-based)
- **Kill Streak**: Current combo (visual only, affects scoring)
- **Survival Time**: Total seconds elapsed

### Tracked (Not Displayed Yet)

- `time_survived`: Total game time
- `max_consecutive_kills`: Best streak
- `difficulty_multiplier`: Current difficulty rating

---

## Game Loop Flow

```
Each Frame:
  1. Update survival time
  2. Check player health (game-over if ≤ 0)
  3. Process damage events → break kill streak
  4. Count enemy deaths → add score + streak
  5. Advance wave progress (30s / wave)
  6. If wave complete: increment wave, reset progress, update difficulty
  7. Render game state to UI
```

---

## Boss Waves (Every 5th Wave)

**Wave 5, 10, 15, 20, ...**

**Boss Characteristics:**
- `enemies_for_wave_adjusted()` includes 40% health boost
- Speed increased by 30%
- Marked with `is_boss_wave()` flag (for future visual distinction)

**Future Enhancements:**
- Boss enemy type with unique attack patterns
- Boss health bar (shared across multiple boss entities)
- Unique sound effects / visual effects
- Boss-specific item drops

---

## Future Mechanics (Roadmap)

### Not in MVP (after workers deliver Jobs #1-6)

**Power-Ups:**
- Speed boost (temporary)
- Damage amplifier (temporary)
- Shield (temporary health)
- Fire rate increase (temporary)

**Player Progression:**
- Experience points (XP)
- Level system
- Unlockable abilities
- Weapon upgrades

**Advanced Scoring:**
- Combo bonuses (streak multiplier)
- Difficulty bonuses (survive later waves = more points)
- Time bonuses (fast clears)
- Achievement bonuses (hidden challenges)

**Game Modes:**
- Survival (unlimited waves)
- Timed (30 minutes endurance)
- Wave challenge (reach wave N)
- Score attack (highest score in 5 minutes)

**Visuals:**
- Enemy status effects (frozen, burning, etc.)
- Hit feedback (enemy knockback, hit markers)
- Particle effects (death explosions)
- Screen shake on major events

---

## Configuration (YAML, Job #4)

All values will be YAML-configurable in Job #4:

```yaml
waves:
  base_enemies: 20
  enemy_health_start: 30.0
  enemy_speed_start: 60.0
  wave_duration_start: 30.0
  difficulty_multiplier: 0.1
  
scoring:
  kill_points: 1
  streak_cap: 5.0
  streak_increment: 50
  
bosses:
  health_multiplier: 1.4
  speed_multiplier: 1.3
  interval: 5
```

---

## Performance Targets

**Scalability with Mechanics:**
- Wave 1: 20 enemies, 100 projectiles → 60+ FPS ✓
- Wave 10: 120 enemies, 500 projectiles → 60+ FPS (with Job #2 pooling)
- Late game (Wave 30+): 200+ enemies, 1000+ projectiles → 60+ FPS (with all 6 jobs)

---

## Integration with Architecture

**Where Game Mechanics Live:**

| Module | Responsibility |
|--------|-----------------|
| `game/state.rs` | Wave progression, scoring, difficulty multipliers |
| `game/waves.rs` | Update spawner parameters based on state |
| `systems/spawning.rs` | Enemy spawn timing and batching |
| `events.rs` | DamageEvent, DeathEvent (trigger streaks) |
| `game/ui.rs` | Display metrics to player |
| `game/input.rs` | Player controls (move, shoot) |
| `game/projectiles.rs` | Projectile spawn rate, damage |

---

## Testing Progression

**Manual Testing Checklist:**

- [ ] Wave 1 completes in ~30 seconds
- [ ] Wave 2 harder than Wave 1 (more enemies, faster)
- [ ] Enemies spawn continuously (not all at once)
- [ ] Score increments per enemy killed
- [ ] Kill streak resets when player takes damage
- [ ] Boss wave appears at Wave 5, 10, 15
- [ ] Game-over triggers when player health ≤ 0
- [ ] Survival time tracks correctly
- [ ] FPS stays 60+ through Wave 5

---

## Known Limitations

1. **No visual feedback for streaks** (yet)
2. **No boss enemy type** (all enemies same)
3. **No power-ups** (score only progression)
4. **No player leveling** (starting stats fixed)
5. **Difficulty plateaus at Wave 30+** (enemies cap at 300 count)

---

**Next**: Workers implement Jobs #1-6 to unlock scaling to 1000+ entities and advanced rendering.
