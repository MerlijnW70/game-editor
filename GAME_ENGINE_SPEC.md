## Vampire Survivors-Inspired Game Engine: Technical Foundation Research

### Objective
Research and document the technical foundation for a world-class game engine designed for a Vampire Survivors-style game (fast-paced top-down 2D action with hundreds of simultaneous entities).

### Deliverable Format
A single markdown document (RESEARCH.md) covering:

1. **Architecture Overview** (20%)
   - Recommended game engine choice (custom, Godot, Bevy, Unreal, etc.) with justification
   - Core systems needed (rendering, physics, entity management, audio)
   - Scalability for 100+ entities on screen simultaneously

2. **Entity & Component System** (25%)
   - Recommended ECS/OOP approach for player, enemies, projectiles, effects
   - How to efficiently manage thousands of operations per frame
   - Memory layout optimization for cache locality

3. **Rendering Pipeline** (20%)
   - 2D rendering optimization for fast frame rates (60+ fps)
   - Sprite batching, layer sorting, particle effects
   - Screen space vs world space considerations

4. **Physics & Collision** (15%)
   - Efficient collision detection for 100+ entities
   - Spatial partitioning (quadtree, grid, broadphase/narrowphase)
   - Performance benchmarks for expected entity counts

5. **Performance Targets & Tooling** (10%)
   - Target specs (resolution, framerate, target devices)
   - Profiling and optimization tools
   - Memory budget recommendations

6. **Quick Start Implementation Path** (10%)
   - Minimal viable prototype tech stack
   - Estimated development timeline
   - Key technical risks and mitigation

### Acceptance Criteria
- Document is well-researched, technical, and actionable
- Includes at least 3 viable engine options with pros/cons
- Provides specific optimization strategies (not generic)
- References real performance data where applicable
- Formatted as valid markdown with clear sections and code examples
- 3000-5000 words of substantive technical content

### Evaluation Method
Judge will verify:
1. Technical depth and accuracy
2. Practicality for Vampire Survivors-style gameplay
3. Completeness of all 6 sections
4. Quality of reasoning and tradeoff analysis
