//! Grid-based spatial partitioning for efficient collision queries.
//!
//! Uses a fixed grid with dynamic entity updates. Optimized for 100+ entities
//! with < 0.5ms per frame updates.

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Grid cell size in world units. Tuned for typical collision radii (10-30 units).
/// Larger cells reduce cell-count overhead; smaller cells reduce collision checks per cell.
const GRID_CELL_SIZE: f32 = 50.0;

/// Grid-based spatial hash. Assigns entities to cells based on position.
pub struct SpatialGrid {
    /// Map from cell key to set of entities in that cell
    cells: HashMap<(i32, i32), Vec<Entity>>,
    /// Map from entity to its grid cells (for quick update/removal)
    entity_cells: HashMap<Entity, HashSet<(i32, i32)>>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }

    /// Get grid cell key from world position
    #[inline]
    fn cell_key(pos: Vec2) -> (i32, i32) {
        (
            (pos.x / GRID_CELL_SIZE).floor() as i32,
            (pos.y / GRID_CELL_SIZE).floor() as i32,
        )
    }

    /// Get all cells overlapped by a circle at `pos` with `radius`
    fn cells_for_circle(pos: Vec2, radius: f32) -> Vec<(i32, i32)> {
        let min_x = ((pos.x - radius) / GRID_CELL_SIZE).floor() as i32;
        let max_x = ((pos.x + radius) / GRID_CELL_SIZE).floor() as i32;
        let min_y = ((pos.y - radius) / GRID_CELL_SIZE).floor() as i32;
        let max_y = ((pos.y + radius) / GRID_CELL_SIZE).floor() as i32;

        let mut cells = Vec::new();
        for gx in min_x..=max_x {
            for gy in min_y..=max_y {
                cells.push((gx, gy));
            }
        }
        cells
    }

    /// Insert or move entity to its new position
    pub fn update_entity(&mut self, entity: Entity, pos: Vec2, radius: f32) {
        // Remove from old cells
        if let Some(old_cells) = self.entity_cells.remove(&entity) {
            for cell in old_cells {
                if let Some(entities) = self.cells.get_mut(&cell) {
                    entities.retain(|e| *e != entity);
                }
            }
        }

        // Add to new cells
        let new_cells: HashSet<_> = Self::cells_for_circle(pos, radius).into_iter().collect();
        for cell in &new_cells {
            self.cells
                .entry(*cell)
                .or_insert_with(Vec::new)
                .push(entity);
        }
        self.entity_cells.insert(entity, new_cells);
    }

    /// Remove entity from grid
    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(cells) = self.entity_cells.remove(&entity) {
            for cell in cells {
                if let Some(entities) = self.cells.get_mut(&cell) {
                    entities.retain(|e| *e != entity);
                }
            }
        }
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }

    /// Get all entities potentially colliding with the circle at `pos` with `radius`
    pub fn query_circle(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let cells = Self::cells_for_circle(pos, radius);
        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for cell in cells {
            if let Some(entities) = self.cells.get(&cell) {
                for &entity in entities {
                    if seen.insert(entity) {
                        result.push(entity);
                    }
                }
            }
        }
        result
    }

    /// Get all entities in axis-aligned bounding box
    pub fn query_aabb(&self, min: Vec2, max: Vec2) -> Vec<Entity> {
        let min_cell = Self::cell_key(min);
        let max_cell = Self::cell_key(max);

        let mut result = Vec::new();
        let mut seen = HashSet::new();

        for gx in min_cell.0..=max_cell.0 {
            for gy in min_cell.1..=max_cell.1 {
                if let Some(entities) = self.cells.get(&(gx, gy)) {
                    for &entity in entities {
                        if seen.insert(entity) {
                            result.push(entity);
                        }
                    }
                }
            }
        }
        result
    }

    /// Rebuild grid from scratch (useful after many updates). Usually not needed.
    pub fn rebuild(&mut self) {
        // Keep the current state; this is a placeholder for future optimization
    }

    /// Get statistics for debugging
    pub fn stats(&self) -> SpatialGridStats {
        SpatialGridStats {
            cells_count: self.cells.len(),
            entities_count: self.entity_cells.len(),
            avg_entities_per_cell: if self.cells.is_empty() {
                0.0
            } else {
                let total: usize = self.cells.values().map(|v| v.len()).sum();
                total as f32 / self.cells.len() as f32
            },
        }
    }
}

impl Default for SpatialGrid {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the spatial grid
#[derive(Debug, Clone, Copy)]
pub struct SpatialGridStats {
    pub cells_count: usize,
    pub entities_count: usize,
    pub avg_entities_per_cell: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_insertion() {
        let mut grid = SpatialGrid::new();
        let e1 = Entity::from_bits(1);
        grid.update_entity(e1, Vec2::ZERO, 10.0);

        let entities = grid.query_circle(Vec2::ZERO, 10.0);
        assert!(entities.contains(&e1));
    }

    #[test]
    fn test_grid_spatial_separation() {
        let mut grid = SpatialGrid::new();
        let e1 = Entity::from_bits(1);
        let e2 = Entity::from_bits(2);

        grid.update_entity(e1, Vec2::ZERO, 10.0);
        grid.update_entity(e2, Vec2::new(200.0, 0.0), 10.0);

        let near_e1 = grid.query_circle(Vec2::ZERO, 10.0);
        assert!(near_e1.contains(&e1));
        assert!(!near_e1.contains(&e2));
    }
}
