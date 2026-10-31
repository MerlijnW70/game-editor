# Task: Debug Tools & In-Game Profiler

## Objective
Build **in-game debugging infrastructure** for performance profiling, entity inspection, and real-time visualization of engine metrics.

## Why This Matters
- Current: No visibility into frame time breakdown (rendering vs ECS vs collision)
- Target: Toggle debug overlay to see FPS, draw calls, system times, entity counts
- Essential for tuning performance and diagnosing bottlenecks

## Technical Requirements

### 1. Frame Time Profiler
- **Measure each system**: Track duration of `CoreSet::Movement`, `Collision`, `Rendering`, etc.
- **Display**: On-screen overlay showing:
  ```
  FPS: 60.0 | Frame: 16.67ms
  Movement: 2.1ms | Collision: 1.5ms | Render: 8.3ms | Other: 4.8ms
  Entities: 142 | Draw calls: 12 | Spatial grid cells: 256
  ```
- **Toggle**: Press `D` to show/hide debug overlay
- **History**: Optional frame time graph (last 60 frames)

### 2. Entity Inspector
- **Show all entities on-screen**: List each entity with components
- **Filter**: Search by entity ID or component type
- **Live edit**: Click to modify component values (health, position, speed) without recompile
- **Highlight**: Click entity to highlight it in game view

### 3. Spatial Visualization
- **Collision shapes**: Draw collision boxes/circles for all entities
- **Spatial grid**: Show grid cells (if grid-based), or quadtree nodes (if quadtree)
- **Highlight hover**: Show which grid cells overlap with mouse position

### 4. Game Controls
- **Time scale**: Slider or keyboard (1-9 keys) to set time scale (0.0 = pause, 0.5 = slow-mo, 1.0 = normal, 2.0 = fast-forward)
- **Pause**: Spacebar pauses game (time scale = 0)
- **Step frame**: F to advance one frame while paused

### 5. Performance Targets
- **Debug overlay**: < 1ms to render (doesn't impact game performance much)
- **Inspector**: < 0.5ms to gather entity data
- **Profiling overhead**: < 2% total frame time

### 6. Example & Tests
- Example: `examples/debug_tools_demo.rs` with debug overlay enabled
- Test: Verify profiler measures correct system times
- Test: Verify entity inspector lists all entities and components
- Visual test: Screenshot of debug overlay with multiple views

## Deliverable Format
- `src/debug/mod.rs` → Debug system module
- `src/debug/profiler.rs` → Frame time profiler
- `src/debug/inspector.rs` → Entity inspector
- `src/debug/rendering.rs` → Spatial visualization
- `src/systems/debug_system.rs` → Input handling + UI rendering
- Update `src/plugin.rs` to include DebugPlugin
- Example: `examples/debug_tools_demo.rs`
- Tests: `tests/debug_profiler.rs`
- Documentation: `DEBUG_TOOLS.md` (how to use inspector/profiler)

## Acceptance Criteria
1. Code compiles (no warnings)
2. Debug overlay renders correctly (FPS, frame time, system breakdown)
3. Toggle debug view (press D)
4. Entity inspector shows all entities + components
5. Live edit: Modify component values, see changes in game immediately
6. Spatial visualization: Draw collision shapes, grid cells, quadtree nodes
7. Time controls: Pause (Spacebar), slow-mo (1-9 keys), step frame (F)
8. Profiler accuracy: Times match system execution order
9. Performance: Debug overhead < 2% frame time
10. Tests pass (profiler, inspector, visualization)
11. Git commit with clear message

## Payment
**0.10 SOL** (complex visualization + interaction)

## Timeline
- **Estimated**: 8-10 hours (UI rendering + profiling)
- **Deadline**: 7 days from job posting

## Notes
- Use `bevy_ui` for inspector UI
- Render collision shapes via `Gizmos` (built-in to Bevy)
- Keep debug systems separate (don't impact game logic)
- Design for quick iteration: add more metrics/views easily later
- Consider `bevy_editor_pls` plugin as reference implementation
