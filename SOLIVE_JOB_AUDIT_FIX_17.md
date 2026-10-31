# SOLIVE Job: Advanced Collision Shapes

**Budget:** 0.07 SOL  
**Deadline:** 7 days  
**Effort:** 3-4 hours  
**Status:** Phase 5 - Core Feature  
**Depends On:** Jobs #1-3 (spatial grid)

---

## Problem Statement

Currently only circle collisions supported:
- Enemies are circles
- Projectiles are circles
- Obstacles must be circular
- No support for rectangles, polygons, complex shapes
- Can't represent realistic level geometry

Production needs:
- Rectangular obstacles (walls, platforms)
- Polygon collision (complex terrain)
- Compound shapes (multi-part enemies)
- Accurate collision for game design

---

## Solution: Shape-based Collision System

### 1. Collision Shape Types

```rust
use std::f32::consts::PI;

/// Collision shape variants
#[derive(Component, Clone, Debug)]
pub enum CollisionShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Polygon { vertices: Vec<Vec2> },
    Capsule { radius: f32, half_length: f32 },
    Compound { shapes: Vec<(Vec2, CollisionShape)> },  // Offset + shape
}

impl CollisionShape {
    /// Get bounding circle (for grid queries)
    pub fn bounding_radius(&self) -> f32 {
        match self {
            CollisionShape::Circle { radius } => *radius,
            CollisionShape::Rectangle { width, height } => {
                (width * width + height * height).sqrt() / 2.0
            }
            CollisionShape::Polygon { vertices } => {
                vertices
                    .iter()
                    .map(|v| v.length())
                    .fold(0.0_f32, f32::max)
            }
            CollisionShape::Capsule { radius, half_length } => {
                (radius * radius + half_length * half_length).sqrt()
            }
            CollisionShape::Compound { shapes } => {
                shapes
                    .iter()
                    .map(|(offset, shape)| offset.length() + shape.bounding_radius())
                    .fold(0.0_f32, f32::max)
            }
        }
    }
    
    /// Get axis-aligned bounding box
    pub fn aabb(&self) -> (Vec2, Vec2) {
        match self {
            CollisionShape::Circle { radius } => {
                (Vec2::new(-radius, -radius), Vec2::new(radius, radius))
            }
            CollisionShape::Rectangle { width, height } => {
                let hw = width / 2.0;
                let hh = height / 2.0;
                (Vec2::new(-hw, -hh), Vec2::new(hw, hh))
            }
            _ => {
                let r = self.bounding_radius();
                (Vec2::new(-r, -r), Vec2::new(r, r))
            }
        }
    }
}

// Helper constructors
impl CollisionShape {
    pub fn circle(radius: f32) -> Self {
        CollisionShape::Circle { radius }
    }
    
    pub fn rect(width: f32, height: f32) -> Self {
        CollisionShape::Rectangle { width, height }
    }
    
    pub fn polygon(vertices: Vec<Vec2>) -> Self {
        CollisionShape::Polygon { vertices }
    }
    
    pub fn capsule(radius: f32, length: f32) -> Self {
        CollisionShape::Capsule {
            radius,
            half_length: length / 2.0,
        }
    }
}
```

### 2. Collision Detection

```rust
pub struct CollisionDetector;

impl CollisionDetector {
    /// Circle-to-circle collision
    pub fn circle_circle(c1: Vec2, r1: f32, c2: Vec2, r2: f32) -> bool {
        c1.distance_squared(c2) < (r1 + r2) * (r1 + r2)
    }
    
    /// Circle-to-rectangle collision
    pub fn circle_rect(
        circle_pos: Vec2,
        circle_radius: f32,
        rect_pos: Vec2,
        rect_width: f32,
        rect_height: f32,
    ) -> bool {
        let dx = (circle_pos.x - rect_pos.x).abs();
        let dy = (circle_pos.y - rect_pos.y).abs();
        
        if dx > rect_width / 2.0 + circle_radius {
            return false;
        }
        if dy > rect_height / 2.0 + circle_radius {
            return false;
        }
        
        if dx <= rect_width / 2.0 || dy <= rect_height / 2.0 {
            return true;
        }
        
        let corner_x = dx - rect_width / 2.0;
        let corner_y = dy - rect_height / 2.0;
        corner_x * corner_x + corner_y * corner_y <= circle_radius * circle_radius
    }
    
    /// Circle-to-polygon collision (using SAT)
    pub fn circle_polygon(
        circle_pos: Vec2,
        circle_radius: f32,
        polygon_pos: Vec2,
        vertices: &[Vec2],
    ) -> bool {
        // Separating Axis Theorem: check distance to each edge
        for i in 0..vertices.len() {
            let v1 = vertices[i] + polygon_pos;
            let v2 = vertices[(i + 1) % vertices.len()] + polygon_pos;
            
            // Distance from circle to line segment
            let edge = v2 - v1;
            let to_circle = circle_pos - v1;
            let t = (to_circle.dot(edge) / edge.length_squared()).clamp(0.0, 1.0);
            let closest = v1 + edge * t;
            
            if circle_pos.distance(closest) <= circle_radius {
                return true;
            }
        }
        
        false
    }
    
    /// Polygon-to-polygon collision (SAT)
    pub fn polygon_polygon(
        pos_a: Vec2,
        vertices_a: &[Vec2],
        pos_b: Vec2,
        vertices_b: &[Vec2],
    ) -> bool {
        // Check axes from both polygons
        for vertices in [vertices_a, vertices_b] {
            for i in 0..vertices.len() {
                let v1 = vertices[i];
                let v2 = vertices[(i + 1) % vertices.len()];
                let edge = v2 - v1;
                
                // Normal to edge (perpendicular)
                let normal = Vec2::new(-edge.y, edge.x).normalize();
                
                // Project both polygons onto normal
                let mut proj_a_min = f32::MAX;
                let mut proj_a_max = f32::MIN;
                for v in vertices_a {
                    let proj = (v + pos_a).dot(normal);
                    proj_a_min = proj_a_min.min(proj);
                    proj_a_max = proj_a_max.max(proj);
                }
                
                let mut proj_b_min = f32::MAX;
                let mut proj_b_max = f32::MIN;
                for v in vertices_b {
                    let proj = (v + pos_b).dot(normal);
                    proj_b_min = proj_b_min.min(proj);
                    proj_b_max = proj_b_max.max(proj);
                }
                
                // Check for gap on this axis
                if proj_a_max < proj_b_min || proj_b_max < proj_a_min {
                    return false;  // Found separating axis
                }
            }
        }
        
        true  // No separating axis found
    }
    
    /// Dispatch collision detection based on shapes
    pub fn detect_collision(
        pos_a: Vec2,
        shape_a: &CollisionShape,
        pos_b: Vec2,
        shape_b: &CollisionShape,
    ) -> bool {
        match (shape_a, shape_b) {
            (CollisionShape::Circle { radius: r_a }, CollisionShape::Circle { radius: r_b }) => {
                Self::circle_circle(pos_a, *r_a, pos_b, *r_b)
            }
            (
                CollisionShape::Circle { radius },
                CollisionShape::Rectangle { width, height },
            ) => Self::circle_rect(pos_a, *radius, pos_b, *width, *height),
            (
                CollisionShape::Rectangle { width, height },
                CollisionShape::Circle { radius },
            ) => Self::circle_rect(pos_b, *radius, pos_a, *width, *height),
            (
                CollisionShape::Circle { radius },
                CollisionShape::Polygon { vertices },
            ) => Self::circle_polygon(pos_a, *radius, pos_b, vertices),
            (
                CollisionShape::Polygon { vertices },
                CollisionShape::Circle { radius },
            ) => Self::circle_polygon(pos_b, *radius, pos_a, vertices),
            (
                CollisionShape::Polygon { vertices: v_a },
                CollisionShape::Polygon { vertices: v_b },
            ) => Self::polygon_polygon(pos_a, v_a, pos_b, v_b),
            // Capsule collisions (treat as circle for simplicity)
            (
                CollisionShape::Capsule { radius, .. },
                other,
            ) => {
                let temp_circle = CollisionShape::Circle { radius: *radius };
                Self::detect_collision(pos_a, &temp_circle, pos_b, other)
            }
            // Compound shapes: test all sub-shapes
            (
                CollisionShape::Compound { shapes },
                other,
            ) => {
                shapes.iter().any(|(offset, shape)| {
                    Self::detect_collision(pos_a + offset, shape, pos_b, other)
                })
            }
            _ => false,  // Unsupported combination
        }
    }
}
```

### 3. Spatial Grid with Shape Support

```rust
pub struct SpatialGridShapes {
    grid: HashMap<(i32, i32), Vec<Entity>>,
    entity_shapes: HashMap<Entity, CollisionShape>,
    entity_positions: HashMap<Entity, Vec2>,
}

impl SpatialGridShapes {
    pub fn new() -> Self {
        Self {
            grid: HashMap::new(),
            entity_shapes: HashMap::new(),
            entity_positions: HashMap::new(),
        }
    }
    
    pub fn update_entity(
        &mut self,
        entity: Entity,
        pos: Vec2,
        shape: &CollisionShape,
    ) {
        // Use bounding radius for grid placement
        let radius = shape.bounding_radius();
        
        self.entity_shapes.insert(entity, shape.clone());
        self.entity_positions.insert(entity, pos);
        
        // Place in grid cells covered by bounding circle
        let min_x = ((pos.x - radius) / 50.0).floor() as i32;
        let max_x = ((pos.x + radius) / 50.0).floor() as i32;
        let min_y = ((pos.y - radius) / 50.0).floor() as i32;
        let max_y = ((pos.y + radius) / 50.0).floor() as i32;
        
        for gx in min_x..=max_x {
            for gy in min_y..=max_y {
                self.grid.entry((gx, gy))
                    .or_insert_with(Vec::new)
                    .push(entity);
            }
        }
    }
    
    pub fn query_circle(
        &self,
        pos: Vec2,
        radius: f32,
    ) -> Vec<Entity> {
        let min_x = ((pos.x - radius) / 50.0).floor() as i32;
        let max_x = ((pos.x + radius) / 50.0).floor() as i32;
        let min_y = ((pos.y - radius) / 50.0).floor() as i32;
        let max_y = ((pos.y + radius) / 50.0).floor() as i32;
        
        let mut results = Vec::new();
        let mut seen = HashSet::new();
        
        for gx in min_x..=max_x {
            for gy in min_y..=max_y {
                if let Some(entities) = self.grid.get(&(gx, gy)) {
                    for &entity in entities {
                        if seen.insert(entity) {
                            results.push(entity);
                        }
                    }
                }
            }
        }
        
        results
    }
}
```

### 4. Collision Detection System

```rust
pub fn detect_collisions_with_shapes(
    mut grid: ResMut<SpatialGridShapes>,
    query: Query<(Entity, &Transform2D, &CollisionShape)>,
    mut collision_events: EventWriter<CollisionEvent>,
) {
    let mut entities: Vec<_> = query.iter().collect();
    
    for i in 0..entities.len() {
        let (entity_a, pos_a, shape_a) = entities[i];
        let nearby = grid.query_circle(pos_a.position, shape_a.bounding_radius());
        
        for entity_b in nearby {
            if entity_a >= entity_b {
                continue;  // Avoid duplicates
            }
            
            if let Ok((_, pos_b, shape_b)) = query.get(entity_b) {
                if CollisionDetector::detect_collision(
                    pos_a.position,
                    shape_a,
                    pos_b.position,
                    shape_b,
                ) {
                    collision_events.send(CollisionEvent {
                        entity_a,
                        entity_b,
                        overlap: 0.0,  // TODO: calculate
                    });
                }
            }
        }
    }
}
```

---

## Acceptance Criteria

✅ `CollisionShape` enum with 5+ variants  
✅ `circle_circle()` detection  
✅ `circle_rect()` detection  
✅ `circle_polygon()` detection (SAT)  
✅ `polygon_polygon()` detection (SAT)  
✅ `CollisionDetector` dispatcher  
✅ `SpatialGridShapes` integration  
✅ Bounding radius calculation  
✅ AABB calculation  
✅ All 6 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_circle_circle() {
    assert!(CollisionDetector::circle_circle(
        Vec2::ZERO,
        5.0,
        Vec2::new(8.0, 0.0),
        5.0
    ));
    
    assert!(!CollisionDetector::circle_circle(
        Vec2::ZERO,
        5.0,
        Vec2::new(15.0, 0.0),
        5.0
    ));
}

#[test]
fn test_circle_rect() {
    assert!(CollisionDetector::circle_rect(
        Vec2::new(5.0, 0.0),  // Circle at (5, 0)
        5.0,                   // Radius 5
        Vec2::ZERO,            // Rect at (0, 0)
        10.0,                  // Width 10
        10.0                   // Height 10
    ));
}

#[test]
fn test_bounding_radius() {
    let circle = CollisionShape::circle(10.0);
    assert_eq!(circle.bounding_radius(), 10.0);
    
    let rect = CollisionShape::rect(20.0, 20.0);
    assert!((rect.bounding_radius() - 14.14).abs() < 0.1);  // sqrt(200)/2 ≈ 14.14
}

#[test]
fn test_aabb() {
    let circle = CollisionShape::circle(5.0);
    let (min, max) = circle.aabb();
    
    assert_eq!(min, Vec2::new(-5.0, -5.0));
    assert_eq!(max, Vec2::new(5.0, 5.0));
}

#[test]
fn test_polygon_collision() {
    let square = vec![
        Vec2::new(-5.0, -5.0),
        Vec2::new(5.0, -5.0),
        Vec2::new(5.0, 5.0),
        Vec2::new(-5.0, 5.0),
    ];
    
    // Circle colliding with square
    assert!(CollisionDetector::circle_polygon(
        Vec2::ZERO,  // Center of square
        3.0,
        Vec2::ZERO,
        &square
    ));
}

#[test]
fn test_compound_shape() {
    let compound = CollisionShape::Compound {
        shapes: vec![
            (Vec2::ZERO, CollisionShape::circle(5.0)),
            (Vec2::new(10.0, 0.0), CollisionShape::circle(5.0)),
        ],
    };
    
    assert_eq!(compound.bounding_radius(), 15.0);  // 10 + 5
}
```

---

## Verification Command

```bash
cargo test test_circle_circle
cargo test test_circle_rect
cargo test test_bounding_radius
cargo test test_aabb
cargo test test_polygon_collision
cargo test test_compound_shape
cargo run --release --example integrated_demo -- --advanced-shapes
# Should see varied collision shapes: rectangles, polygons working
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_ADVANCED_SHAPES
```

Expected: ✅ All shape types working, SAT collision accurate, grid integration seamless

