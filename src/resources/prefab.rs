use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Template for spawning entities with predefined components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrefabTemplate {
    pub name: String,
    pub entity_type: EntityType,
    pub transform: TransformData,
    pub visual: VisualData,
    pub gameplay: GameplayData,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EntityType {
    Enemy,
    Projectile,
    Effect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformData {
    pub position: (f32, f32),
    pub rotation: f32,
    pub scale: (f32, f32),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualData {
    pub size: f32,
    pub layer: u32,
    pub color: (f32, f32, f32, f32), // RGBA
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameplayData {
    pub health: f32,
    pub speed: f32,
    pub radius: f32,
    pub lifetime: Option<f32>,
}

/// Manager for all prefab templates
#[derive(Resource, Debug)]
pub struct PrefabLibrary {
    templates: HashMap<String, PrefabTemplate>,
}

impl PrefabLibrary {
    pub fn new() -> Self {
        Self {
            templates: HashMap::new(),
        }
    }

    /// Load prefab from RON file
    pub fn load_template<P: AsRef<Path>>(
        &mut self,
        name: String,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let template: PrefabTemplate = ron::from_str(&content)?;
        self.templates.insert(name, template);
        Ok(())
    }

    /// Load all prefabs from a directory
    pub fn load_directory<P: AsRef<Path>>(
        &mut self,
        dir_path: P,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let dir = fs::read_dir(dir_path)?;
        let mut count = 0;

        for entry in dir {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|ext| ext == "ron").unwrap_or(false) {
                if let Some(filename) = path.file_stem() {
                    let name = filename.to_string_lossy().to_string();
                    if self.load_template(name.clone(), &path).is_ok() {
                        count += 1;
                    }
                }
            }
        }

        Ok(count)
    }

    /// Get template by name
    pub fn get(&self, name: &str) -> Option<&PrefabTemplate> {
        self.templates.get(name)
    }

    /// Get all template names
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Save template to RON file
    pub fn save_template<P: AsRef<Path>>(
        &self,
        name: &str,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(template) = self.get(name) {
            let content = ron::ser::to_string_pretty(template, Default::default())?;
            fs::write(path, content)?;
            Ok(())
        } else {
            Err("Template not found".into())
        }
    }
}

impl Default for PrefabLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker component for entities spawned from prefabs
#[derive(Component, Debug, Clone)]
pub struct SpawnedFromPrefab {
    pub template_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefab_creation() {
        let prefab = PrefabTemplate {
            name: "basic_enemy".to_string(),
            entity_type: EntityType::Enemy,
            transform: TransformData {
                position: (0.0, 0.0),
                rotation: 0.0,
                scale: (1.0, 1.0),
            },
            visual: VisualData {
                size: 10.0,
                layer: 1,
                color: (1.0, 0.0, 0.0, 1.0),
            },
            gameplay: GameplayData {
                health: 30.0,
                speed: 60.0,
                radius: 10.0,
                lifetime: None,
            },
        };

        assert_eq!(prefab.name, "basic_enemy");
        assert_eq!(prefab.entity_type, EntityType::Enemy);
        assert_eq!(prefab.gameplay.health, 30.0);
    }

    #[test]
    fn test_prefab_serialization() {
        let prefab = PrefabTemplate {
            name: "test".to_string(),
            entity_type: EntityType::Enemy,
            transform: TransformData {
                position: (10.0, 20.0),
                rotation: 0.5,
                scale: (1.0, 1.0),
            },
            visual: VisualData {
                size: 10.0,
                layer: 1,
                color: (1.0, 0.0, 0.0, 1.0),
            },
            gameplay: GameplayData {
                health: 30.0,
                speed: 60.0,
                radius: 10.0,
                lifetime: None,
            },
        };

        let ron_str = ron::to_string(&prefab).unwrap();
        assert!(ron_str.contains("basic_enemy") || ron_str.contains("test"));
    }

    #[test]
    fn test_prefab_library() {
        let mut library = PrefabLibrary::new();
        let prefab = PrefabTemplate {
            name: "enemy".to_string(),
            entity_type: EntityType::Enemy,
            transform: TransformData {
                position: (0.0, 0.0),
                rotation: 0.0,
                scale: (1.0, 1.0),
            },
            visual: VisualData {
                size: 10.0,
                layer: 1,
                color: (1.0, 0.0, 0.0, 1.0),
            },
            gameplay: GameplayData {
                health: 30.0,
                speed: 60.0,
                radius: 10.0,
                lifetime: None,
            },
        };

        // Manually add template (normally loaded from file)
        library.templates.insert("enemy".to_string(), prefab);

        assert!(library.get("enemy").is_some());
        assert_eq!(library.list_templates().len(), 1);
    }
}
