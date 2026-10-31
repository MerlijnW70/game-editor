//! Quadtree-based spatial partitioning for efficient collision queries.
//!
//! Replaces the grid-based approach with adaptive quadtree subdivision.
//! Provides O(log n) query performance vs O(n) grid worst-case.
//! Optimized for dynamic scenes with varying entity density.

use bevy::prelude::*;
use std::collections::HashMap;

const MAX_ENTITIES_PER_NODE: usize = 8;
const MIN_NODE_SIZE: f32 = 10.0;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec2,
    pub max: Vec2,
}

impl AABB {
    pub fn new(min: Vec2, max: Vec2) -> Self {
        Self { min, max }
    }

    pub fn center(&self) -> Vec2 {
        (self.min + self.max) / 2.0
    }

    pub fn size(&self) -> Vec2 {
        self.max - self.min
    }

    /// Check if point is inside AABB
    pub fn contains_point(&self, point: Vec2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if circle overlaps with AABB
    pub fn overlaps_circle(&self, center: Vec2, radius: f32) -> bool {
        let closest = Vec2::new(
            center.x.max(self.min.x).min(self.max.x),
            center.y.max(self.min.y).min(self.max.y),
        );
        center.distance(closest) <= radius
    }

    /// Split AABB into 4 quadrants
    pub fn split(&self) -> [AABB; 4] {
        let center = self.center();
        [
            // Top-right
            AABB::new(Vec2::new(center.x, center.y), self.max),
            // Top-left
            AABB::new(Vec2::new(self.min.x, center.y), Vec2::new(center.x, self.max.y)),
            // Bottom-left
            AABB::new(self.min, center),
            // Bottom-right
            AABB::new(Vec2::new(center.x, self.min.y), Vec2::new(self.max.x, center.y)),
        ]
    }
}

struct QuadtreeNode {
    bounds: AABB,
    entities: Vec<(Entity, Vec2, f32)>, // entity, pos, radius
    children: Option<Box<[QuadtreeNode; 4]>>,
}

impl QuadtreeNode {
    fn new(bounds: AABB) -> Self {
        Self {
            bounds,
            entities: Vec::new(),
            children: None,
        }
    }

    #[allow(dead_code)]
    fn is_leaf(&self) -> bool {
        self.children.is_none()
    }

    fn can_split(&self) -> bool {
        let size = self.bounds.size().min_element();
        size > MIN_NODE_SIZE
    }

    fn split(&mut self) {
        if !self.can_split() {
            return;
        }

        let bounds = self.bounds.split();
        let mut children: [QuadtreeNode; 4] = [
            QuadtreeNode::new(bounds[0]),
            QuadtreeNode::new(bounds[1]),
            QuadtreeNode::new(bounds[2]),
            QuadtreeNode::new(bounds[3]),
        ];

        // Redistribute entities to children
        for (entity, pos, radius) in self.entities.drain(..) {
            for child in &mut children {
                if child.bounds.overlaps_circle(pos, radius) {
                    child.entities.push((entity, pos, radius));
                }
            }
        }

        self.children = Some(Box::new(children));
    }

    fn insert(&mut self, entity: Entity, pos: Vec2, radius: f32) {
        if !self.bounds.overlaps_circle(pos, radius) {
            return;
        }

        if let Some(children) = &mut self.children {
            // Distribute to children
            for child in children.iter_mut() {
                child.insert(entity, pos, radius);
            }
        } else {
            // Leaf node: add entity
            self.entities.push((entity, pos, radius));

            // Split if needed
            if self.entities.len() > MAX_ENTITIES_PER_NODE {
                self.split();
                // Re-insert after split
                if let Some(children) = &mut self.children {
                    if let Some((e, p, r)) = self.entities.pop() {
                        for child in children.iter_mut() {
                            child.insert(e, p, r);
                        }
                    }
                }
            }
        }
    }

    fn query(&self, pos: Vec2, radius: f32, result: &mut Vec<Entity>) {
        if !self.bounds.overlaps_circle(pos, radius) {
            return;
        }

        // Check entities in this node
        for (entity, _, _) in &self.entities {
            result.push(*entity);
        }

        // Check children
        if let Some(children) = &self.children {
            for child in children.iter() {
                child.query(pos, radius, result);
            }
        }
    }

    fn remove(&mut self, entity: Entity) -> bool {
        // Try to remove from this node
        if self.entities.iter().position(|(e, _, _)| *e == entity).is_some() {
            self.entities.retain(|(e, _, _)| *e != entity);
            return true;
        }

        // Try children
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.remove(entity) {
                    return true;
                }
            }
        }

        false
    }

    fn clear(&mut self) {
        self.entities.clear();
        self.children = None;
    }
}

/// Quadtree-based spatial index with same API as grid
pub struct QuadtreeIndex {
    root: QuadtreeNode,
    entity_positions: HashMap<Entity, (Vec2, f32)>, // Track entity pos/radius for updates
}

impl QuadtreeIndex {
    pub fn new(bounds: AABB) -> Self {
        Self {
            root: QuadtreeNode::new(bounds),
            entity_positions: HashMap::new(),
        }
    }

    /// Insert or move entity to its new position
    pub fn update_entity(&mut self, entity: Entity, pos: Vec2, radius: f32) {
        // Remove old position
        if self.entity_positions.contains_key(&entity) {
            self.root.remove(entity);
        }

        // Insert at new position
        self.root.insert(entity, pos, radius);
        self.entity_positions.insert(entity, (pos, radius));
    }

    /// Remove entity from index
    pub fn remove_entity(&mut self, entity: Entity) {
        self.root.remove(entity);
        self.entity_positions.remove(&entity);
    }

    /// Clear all entities
    pub fn clear(&mut self) {
        self.root.clear();
        self.entity_positions.clear();
    }

    /// Get all entities potentially colliding with circle
    pub fn query_circle(&self, pos: Vec2, radius: f32) -> Vec<Entity> {
        let mut result = Vec::new();
        self.root.query(pos, radius, &mut result);
        // Remove duplicates (entities may appear in multiple overlapping nodes)
        result.sort();
        result.dedup();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aabb_contains_point() {
        let aabb = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(aabb.contains_point(Vec2::new(5.0, 5.0)));
        assert!(!aabb.contains_point(Vec2::new(11.0, 5.0)));
    }

    #[test]
    fn test_aabb_overlaps_circle() {
        let aabb = AABB::new(Vec2::new(0.0, 0.0), Vec2::new(10.0, 10.0));
        assert!(aabb.overlaps_circle(Vec2::new(5.0, 5.0), 5.0));
        assert!(!aabb.overlaps_circle(Vec2::new(20.0, 20.0), 1.0));
    }

    #[test]
    fn test_quadtree_insert_query() {
        let bounds = AABB::new(Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
        let mut tree = QuadtreeIndex::new(bounds);

        let e1 = Entity::from_raw(1);
        let e2 = Entity::from_raw(2);

        tree.update_entity(e1, Vec2::new(0.0, 0.0), 5.0);
        tree.update_entity(e2, Vec2::new(50.0, 50.0), 5.0);

        let nearby = tree.query_circle(Vec2::new(0.0, 0.0), 10.0);
        assert!(nearby.contains(&e1));
        assert!(!nearby.contains(&e2));
    }

    #[test]
    fn test_quadtree_remove() {
        let bounds = AABB::new(Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
        let mut tree = QuadtreeIndex::new(bounds);

        let e1 = Entity::from_raw(1);
        tree.update_entity(e1, Vec2::new(0.0, 0.0), 5.0);

        let results = tree.query_circle(Vec2::new(0.0, 0.0), 10.0);
        assert!(results.contains(&e1));

        tree.remove_entity(e1);
        let results = tree.query_circle(Vec2::new(0.0, 0.0), 10.0);
        assert!(!results.contains(&e1));
    }
}
