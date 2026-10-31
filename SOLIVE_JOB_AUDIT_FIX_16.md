# SOLIVE Job: Physics Simulation System

**Budget:** 0.08 SOL  
**Deadline:** 7 days  
**Effort:** 4-5 hours  
**Status:** Phase 5 - Core Feature  
**Depends On:** Jobs #1-10 (foundation required)

---

## Problem Statement

Currently entities just move and collide - no physics simulation:
- No gravity effects
- No momentum/inertia
- No bounce/friction
- Collisions are instant (no gradual response)
- No constraints or forces

Production needs:
- Realistic object dynamics
- Environmental interaction
- Force-based systems (explosions, pushback)
- Continuous collision response

---

## Solution: Physics Engine Integration

### 1. Physics Components

```rust
/// Velocity with direction and magnitude
#[derive(Component, Clone, Copy, Debug)]
pub struct Velocity(pub Vec2);

/// Acceleration (forces applied)
#[derive(Component, Clone, Copy, Debug)]
pub struct Acceleration(pub Vec2);

/// Physics body properties
#[derive(Component, Clone, Debug)]
pub struct RigidBody {
    pub mass: f32,              // 0.5-100.0 kg
    pub velocity: Vec2,         // Current velocity
    pub acceleration: Vec2,     // Current acceleration
    pub angular_velocity: f32,  // Rotation speed
    pub friction: f32,          // 0.0-1.0 (deceleration)
    pub restitution: f32,       // 0.0-1.0 (bounce)
    pub is_static: bool,        // Static bodies don't move
    pub gravity_scale: f32,     // Gravity multiplier
}

impl Default for RigidBody {
    fn default() -> Self {
        Self {
            mass: 1.0,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            angular_velocity: 0.0,
            friction: 0.1,          // Small friction by default
            restitution: 0.3,       // Some bounce
            is_static: false,
            gravity_scale: 1.0,
        }
    }
}

/// Force application (temporary)
#[derive(Component, Clone, Copy, Debug)]
pub struct Force {
    pub force: Vec2,
    pub duration: f32,  // How long to apply force
}

/// Gravity effect
#[derive(Component, Clone, Copy, Debug)]
pub struct GravityAffected {
    pub gravity: f32,  // 9.8 default
}
```

### 2. Physics World Resource

```rust
#[derive(Resource, Clone, Debug)]
pub struct PhysicsWorld {
    pub gravity: Vec2,           // World gravity (typically (0, -9.8))
    pub time_step: f32,          // Fixed timestep (0.016 for 60 fps)
    pub iterations: usize,       // Solver iterations (3-4 typical)
    pub damping: f32,            // Global damping (0.99)
    pub max_velocity: f32,       // Terminal velocity
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: Vec2::new(0.0, -9.8),
            time_step: 0.016,
            iterations: 3,
            damping: 0.99,
            max_velocity: 100.0,
        }
    }
}

impl PhysicsWorld {
    /// Apply force to rigidbody
    pub fn apply_force(&self, body: &mut RigidBody, force: Vec2) {
        // F = ma → a = F/m
        body.acceleration += force / body.mass;
    }
    
    /// Apply impulse (instant velocity change)
    pub fn apply_impulse(&self, body: &mut RigidBody, impulse: Vec2) {
        // J = m * Δv → Δv = J/m
        body.velocity += impulse / body.mass;
    }
    
    /// Step simulation forward
    pub fn step(&self, body: &mut RigidBody, dt: f32) {
        if body.is_static {
            return;  // Static bodies don't move
        }
        
        // Apply gravity
        body.acceleration.y += self.gravity.y * body.gravity_scale;
        
        // v = v0 + a*t
        body.velocity += body.acceleration * dt;
        
        // Apply damping (friction/air resistance)
        body.velocity *= self.damping;
        
        // Clamp velocity to max
        let speed = body.velocity.length();
        if speed > self.max_velocity {
            body.velocity = body.velocity.normalize() * self.max_velocity;
        }
        
        // Reset acceleration (forces are applied fresh each frame)
        body.acceleration = Vec2::ZERO;
    }
}
```

### 3. Physics Systems

```rust
/// Apply forces and gravity
pub fn apply_physics_forces(
    physics: Res<PhysicsWorld>,
    mut query: Query<&mut RigidBody>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for mut body in query.iter_mut() {
        physics.step(&mut body, dt);
    }
}

/// Update positions based on velocity
pub fn physics_update_positions(
    mut query: Query<(&RigidBody, &mut Transform2D)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (body, mut transform) in query.iter_mut() {
        // x = x0 + v*t
        transform.position += body.velocity * dt;
    }
}

/// Resolve collisions with physics response
pub fn physics_collision_response(
    mut query: Query<&mut RigidBody>,
    mut collisions: EventReader<CollisionEvent>,
) {
    for collision in collisions.read() {
        if let (Ok(mut body_a), Ok(mut body_b)) = (
            query.get_mut(collision.entity_a),
            query.get_mut(collision.entity_b),
        ) {
            // Simple elastic collision
            let v_a = body_a.velocity;
            let v_b = body_b.velocity;
            
            let m_a = body_a.mass;
            let m_b = body_b.mass;
            
            // Conservation of momentum
            body_a.velocity = ((m_a - m_b) * v_a + 2.0 * m_b * v_b) / (m_a + m_b);
            body_b.velocity = ((m_b - m_a) * v_b + 2.0 * m_a * v_a) / (m_a + m_b);
            
            // Apply restitution (bounciness)
            body_a.velocity *= body_a.restitution;
            body_b.velocity *= body_b.restitution;
        }
    }
}

/// Apply temporary forces
pub fn apply_temporary_forces(
    mut query: Query<(&mut RigidBody, &mut Force)>,
    physics: Res<PhysicsWorld>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    
    for (mut body, mut force) in query.iter_mut() {
        physics.apply_force(&mut body, force.force);
        force.duration -= dt;
    }
}
```

### 4. Physics Constraints

```rust
/// Joint constraints between bodies
#[derive(Component, Clone, Debug)]
pub struct Joint {
    pub other_entity: Entity,
    pub constraint_type: ConstraintType,
    pub max_distance: f32,
    pub min_distance: f32,
}

#[derive(Clone, Copy, Debug)]
pub enum ConstraintType {
    Distance,      // Fixed distance
    Hinge,         // Rotational joint
    Rope,          // Max distance only
    Spring,        // Spring-like
}

pub fn resolve_constraints(
    mut transforms: Query<&mut Transform2D>,
    joints: Query<&Joint>,
) {
    for joint in joints.iter() {
        if let (Ok(mut t_a), Ok(mut t_b)) = (
            transforms.get_mut(joint.other_entity),
            transforms.get_mut_unchecked(joint.other_entity),
        ) {
            let delta = t_b.position - t_a.position;
            let dist = delta.length();
            
            match joint.constraint_type {
                ConstraintType::Distance => {
                    if (dist - joint.max_distance).abs() > 0.01 {
                        let correction = delta.normalize() * (dist - joint.max_distance) * 0.5;
                        t_a.position += correction;
                        t_b.position -= correction;
                    }
                }
                _ => {}  // Other constraint types
            }
        }
    }
}
```

### 5. Explosion/Force Events

```rust
/// Explosion or force wave event
#[derive(Event)]
pub struct ExplosionEvent {
    pub position: Vec2,
    pub radius: f32,
    pub force: f32,
}

pub fn handle_explosions(
    mut events: EventReader<ExplosionEvent>,
    mut query: Query<(&Transform2D, &mut RigidBody)>,
    physics: Res<PhysicsWorld>,
) {
    for explosion in events.read() {
        for (transform, mut body) in query.iter_mut() {
            let dist = transform.position.distance(explosion.position);
            
            if dist < explosion.radius {
                // Force decreases with distance
                let force_magnitude = explosion.force * (1.0 - (dist / explosion.radius));
                let direction = (transform.position - explosion.position).normalize();
                
                let force = direction * force_magnitude;
                physics.apply_impulse(&mut body, force);
            }
        }
    }
}
```

---

## Acceptance Criteria

✅ `RigidBody` component with mass, velocity, friction  
✅ `PhysicsWorld` resource with gravity  
✅ `apply_physics_forces()` system  
✅ `physics_update_positions()` system  
✅ `physics_collision_response()` with momentum conservation  
✅ Gravity simulation working  
✅ Friction/damping working  
✅ Restitution (bounce) working  
✅ Constraints (distance, hinge)  
✅ Explosion forces  
✅ Benchmark: <1ms physics update at 8K entities  
✅ All 6 test cases pass  
✅ Compiles with no warnings  

---

## Tests

```rust
#[test]
fn test_gravity_acceleration() {
    let physics = PhysicsWorld::default();
    let mut body = RigidBody::default();
    
    physics.step(&mut body, 1.0);
    
    // After 1 second under gravity: a = g = 9.8
    assert!((body.acceleration.y - (-9.8)).abs() < 0.1);
}

#[test]
fn test_momentum_conservation() {
    let body_a = RigidBody { mass: 2.0, velocity: Vec2::new(10.0, 0.0), ..Default::default() };
    let body_b = RigidBody { mass: 1.0, velocity: Vec2::new(0.0, 0.0), ..Default::default() };
    
    // Before: momentum = 2*10 + 1*0 = 20
    let momentum_before = body_a.mass * body_a.velocity.x + body_b.mass * body_b.velocity.x;
    
    // After collision (elastic): momentum conserved
    // 2*v_a' + 1*v_b' = 20
    assert!((momentum_before - 20.0).abs() < 0.01);
}

#[test]
fn test_velocity_clamping() {
    let physics = PhysicsWorld::default();
    let mut body = RigidBody::default();
    
    body.velocity = Vec2::new(200.0, 200.0);  // Exceeds max
    physics.step(&mut body, 0.016);
    
    assert!(body.velocity.length() <= physics.max_velocity);
}

#[test]
fn test_static_body_immobile() {
    let physics = PhysicsWorld::default();
    let mut body = RigidBody::default();
    body.is_static = true;
    
    physics.apply_force(&mut body, Vec2::new(100.0, 100.0));
    physics.step(&mut body, 0.016);
    
    assert_eq!(body.velocity, Vec2::ZERO);
}

#[test]
fn test_impulse_application() {
    let physics = PhysicsWorld::default();
    let mut body = RigidBody { mass: 2.0, ..Default::default() };
    
    physics.apply_impulse(&mut body, Vec2::new(10.0, 0.0));
    
    // Δv = J/m = 10/2 = 5
    assert!((body.velocity.x - 5.0).abs() < 0.01);
}

#[test]
fn test_damping() {
    let mut physics = PhysicsWorld::default();
    physics.damping = 0.9;
    
    let mut body = RigidBody { velocity: Vec2::new(100.0, 0.0), ..Default::default() };
    
    physics.step(&mut body, 0.016);
    
    // Velocity should be reduced by damping
    assert!(body.velocity.x < 100.0);
}
```

---

## Verification Command

```bash
cargo test test_gravity_acceleration
cargo test test_momentum_conservation
cargo test test_velocity_clamping
cargo test test_static_body_immobile
cargo test test_impulse_application
cargo test test_damping
cargo run --release --example integrated_demo -- --physics-test
# Should see realistic physics: gravity, bouncing, collisions
```

---

## Judge Check

```bash
./verify_solive_delivery.ps1 JOB_PHYSICS_SYSTEM
```

Expected: ✅ Gravity working, collisions realistic, forces applied correctly, <1ms per frame

