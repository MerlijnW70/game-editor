//! Grid-based spatial partitioning for efficient collision queries.
//!
//! Uses a fixed grid with dynamic entity updates. Optimized for 100+ entities
//! with < 0.5ms per frame updates.

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

/// Grid cell size in world units. Tuned for typical collision radii (10-30 units).
/// Larger cells reduce cell-count overhead; smaller cells reduce collision checks per cell.
const GRID_CELL_SIZE: f32 = 50.0;

/// Why `SpatialGrid::update_entity` refused an entity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SpatialError {
    /// The position has a NaN or infinite component.
    NonFinitePosition {
        /// The rejected position.
        position: Vec2,
    },
    /// The radius is not a finite number greater than zero.
    InvalidRadius {
        /// The rejected radius.
        radius: f32,
    },
    /// The radius exceeds `SpatialGrid::MAX_RADIUS`.
    RadiusTooLarge {
        /// The rejected radius.
        radius: f32,
        /// The largest accepted radius.
        max: f32,
    },
}

impl std::fmt::Display for SpatialError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpatialError::NonFinitePosition { position } => {
                write!(f, "collider position {position} is not finite")
            }
            SpatialError::InvalidRadius { radius } => {
                write!(
                    f,
                    "collider radius {radius} must be a finite number greater than 0"
                )
            }
            SpatialError::RadiusTooLarge { radius, max } => {
                write!(f, "collider radius {radius} exceeds the grid maximum {max}")
            }
        }
    }
}

impl std::error::Error for SpatialError {}

/// Grid-based spatial hash. Assigns entities to cells based on position.
///
/// Cells are keyed by signed coordinates (`floor(pos / cell)`), so negative
/// world positions map to their own cells instead of wrapping.
pub struct SpatialGrid {
    /// Map from cell key to set of entities in that cell
    cells: HashMap<(i32, i32), Vec<Entity>>,
    /// Map from entity to its grid cells (for quick update/removal)
    entity_cells: HashMap<Entity, HashSet<(i32, i32)>>,
}

/// Inclusive range of cell keys covering a rectangle.
#[derive(Debug, Clone, Copy)]
struct CellRange {
    min: (i32, i32),
    max: (i32, i32),
}

impl CellRange {
    fn covering(min: Vec2, max: Vec2) -> Self {
        Self {
            min: SpatialGrid::cell_key(min),
            max: SpatialGrid::cell_key(max),
        }
    }

    /// Number of cells in the range (saturating; ranges can be huge).
    fn len(&self) -> u64 {
        let w = (i64::from(self.max.0) - i64::from(self.min.0) + 1).max(0) as u64;
        let h = (i64::from(self.max.1) - i64::from(self.min.1) + 1).max(0) as u64;
        w.saturating_mul(h)
    }

    fn contains(&self, cell: (i32, i32)) -> bool {
        (self.min.0..=self.max.0).contains(&cell.0) && (self.min.1..=self.max.1).contains(&cell.1)
    }

    fn iter(self) -> impl Iterator<Item = (i32, i32)> {
        (self.min.0..=self.max.0)
            .flat_map(move |gx| (self.min.1..=self.max.1).map(move |gy| (gx, gy)))
    }
}

impl SpatialGrid {
    /// World-space size of one grid cell.
    pub const CELL_SIZE: f32 = GRID_CELL_SIZE;

    /// Largest collider radius the grid accepts: two cells. Bigger colliders
    /// would be inserted into many cells per frame; they belong in a dedicated
    /// structure, not the per-entity hash.
    pub const MAX_RADIUS: f32 = GRID_CELL_SIZE * 2.0;

    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            entity_cells: HashMap::new(),
        }
    }

    /// Get grid cell key from world position. `floor` keeps negative
    /// coordinates in their own cells; out-of-range values saturate.
    #[inline]
    fn cell_key(pos: Vec2) -> (i32, i32) {
        (
            (pos.x / GRID_CELL_SIZE).floor() as i32,
            (pos.y / GRID_CELL_SIZE).floor() as i32,
        )
    }

    /// Checks a collider before it enters the grid.
    pub fn validate_collider(pos: Vec2, radius: f32) -> Result<(), SpatialError> {
        if !pos.is_finite() {
            return Err(SpatialError::NonFinitePosition { position: pos });
        }
        if !(radius.is_finite() && radius > 0.0) {
            return Err(SpatialError::InvalidRadius { radius });
        }
        if radius > Self::MAX_RADIUS {
            return Err(SpatialError::RadiusTooLarge {
                radius,
                max: Self::MAX_RADIUS,
            });
        }
        Ok(())
    }

    /// Inserts or moves `entity` to the circle at `pos` with `radius`.
    ///
    /// An invalid collider is rejected with the reason, and the entity is
    /// removed from the grid so stale cells cannot produce phantom contacts.
    pub fn update_entity(
        &mut self,
        entity: Entity,
        pos: Vec2,
        radius: f32,
    ) -> Result<(), SpatialError> {
        self.remove_entity(entity);
        Self::validate_collider(pos, radius)?;

        let range = CellRange::covering(pos - Vec2::splat(radius), pos + Vec2::splat(radius));
        let new_cells: HashSet<_> = range.iter().collect();
        for cell in &new_cells {
            self.cells.entry(*cell).or_default().push(entity);
        }
        self.entity_cells.insert(entity, new_cells);
        Ok(())
    }

    /// Remove entity from grid
    pub fn remove_entity(&mut self, entity: Entity) {
        if let Some(cells) = self.entity_cells.remove(&entity) {
            for cell in cells {
                if let Some(entities) = self.cells.get_mut(&cell) {
                    entities.retain(|e| *e != entity);
                    if entities.is_empty() {
                        self.cells.remove(&cell);
                    }
                }
            }
        }
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.cells.clear();
        self.entity_cells.clear();
    }

    /// Every entity stored in a cell of `range`, each once.
    ///
    /// Walks whichever is smaller: the cells in the range or the occupied
    /// cells, so a huge query range cannot stall the frame.
    fn collect_range(&self, range: CellRange) -> Vec<Entity> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        let mut take = |entities: &Vec<Entity>| {
            for &entity in entities {
                if seen.insert(entity) {
                    result.push(entity);
                }
            }
        };
        if range.len() <= self.cells.len() as u64 {
            for cell in range.iter() {
                if let Some(entities) = self.cells.get(&cell) {
                    take(entities);
                }
            }
        } else {
            for (cell, entities) in &self.cells {
                if range.contains(*cell) {
                    take(entities);
                }
            }
        }
        result
    }

    /// Entities potentially colliding with the circle at `pos` with `radius`.
    /// A non-finite position or a negative/NaN radius matches nothing.
    pub fn query_circle(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        if !pos.is_finite() || radius.is_nan() || radius < 0.0 {
            return Vec::new();
        }
        self.collect_range(CellRange::covering(
            pos - Vec2::splat(radius),
            pos + Vec2::splat(radius),
        ))
    }

    /// Entities in cells overlapping the box; corners may be in either order.
    /// Non-finite corners match nothing.
    pub fn query_aabb(&self, min: Vec2, max: Vec2) -> Vec<Entity> {
        if !min.is_finite() || !max.is_finite() {
            return Vec::new();
        }
        self.collect_range(CellRange::covering(min.min(max), min.max(max)))
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
        let e1 = Entity::from_raw_u32(1).expect("valid index");
        grid.update_entity(e1, Vec2::ZERO, 10.0)
            .expect("valid collider");

        let entities = grid.query_circle(Vec2::ZERO, 10.0);
        assert!(entities.contains(&e1));
    }

    #[test]
    fn test_grid_spatial_separation() {
        let mut grid = SpatialGrid::new();
        let e1 = Entity::from_raw_u32(1).expect("valid index");
        let e2 = Entity::from_raw_u32(2).expect("valid index");

        grid.update_entity(e1, Vec2::ZERO, 10.0)
            .expect("valid collider");
        grid.update_entity(e2, Vec2::new(200.0, 0.0), 10.0)
            .expect("valid collider");

        let near_e1 = grid.query_circle(Vec2::ZERO, 10.0);
        assert!(near_e1.contains(&e1));
        assert!(!near_e1.contains(&e2));
    }
}
