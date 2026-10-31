use crate::components::{
    Disabled, Health, Layer, Lifetime, Sprite, Team, Transform2D, Velocity,
};
use crate::resources::{PrefabLibrary, prefab::{EntityType, SpawnedFromPrefab}};
use crate::resources::{PoolManager, PoolType};
use bevy::prelude::*;

/// Spawn an entity from a prefab template
pub fn spawn_from_prefab(
    prefab_name: &str,
    position: Vec2,
    library: &PrefabLibrary,
    pool: &mut PoolManager,
    commands: &mut Commands,
) -> Option<Entity> {
    let template = library.get(prefab_name)?;

    // Determine which pool to use
    let pool_type = match template.entity_type {
        EntityType::Enemy => PoolType::Enemy,
        EntityType::Projectile => PoolType::Projectile,
        EntityType::Effect => PoolType::Effect,
    };

    // Acquire entity from pool
    let entity = pool.acquire(pool_type)?;

    // Apply prefab data to entity
    commands.entity(entity).insert((
        Health::new(template.gameplay.health),
        Velocity(Vec2::ZERO),
        Transform2D {
            position,
            rotation: template.transform.rotation,
            scale: Vec2::new(template.transform.scale.0, template.transform.scale.1),
        },
        Layer::new(template.visual.layer as u8, 0),
        Sprite::new("assets/sprites/default.png", Vec2::splat(template.visual.size)),
        Team::Neutral,
        SpawnedFromPrefab {
            template_name: prefab_name.to_string(),
        },
    ));

    // Add lifetime if specified
    if let Some(lifetime) = template.gameplay.lifetime {
        commands.entity(entity).insert(Lifetime::new(lifetime));
    }

    // Add marker component based on type
    match template.entity_type {
        EntityType::Enemy => {
            commands.entity(entity).insert(crate::components::Enemy);
        }
        EntityType::Projectile => {
            commands.entity(entity).insert(crate::components::Projectile);
        }
        EntityType::Effect => {
            // Effects don't need special marker
        }
    }

    // Remove disabled marker to activate
    commands.entity(entity).remove::<Disabled>();

    Some(entity)
}

/// System to load prefab library at startup
pub fn load_prefabs(mut library: ResMut<PrefabLibrary>) {
    match library.load_directory("assets/prefabs") {
        Ok(count) => eprintln!("Loaded {} prefabs", count),
        Err(e) => eprintln!("Failed to load prefabs: {}", e),
    }

    let templates = library.list_templates();
    if !templates.is_empty() {
        eprintln!("Available prefabs: {:?}", templates);
    }
}
