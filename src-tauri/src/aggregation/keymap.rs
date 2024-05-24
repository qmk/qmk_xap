use serde::Serialize;
use specta::Type;

use crate::aggregation::config::LayoutEntry;
use crate::xap::device::KeymapKey;

use super::{Point2D, Point3D};

#[derive(Clone, Debug, Default, Serialize, Type)]
pub struct MappedKeymapKey {
    pub key: KeymapKey,
    pub layout: LayoutEntry,
}

#[derive(Clone, Debug, Serialize, Type)]
pub struct MappedKeymap {
    pub keys: Vec<Vec<Vec<Option<MappedKeymapKey>>>>,
    pub dimensions: Point3D,
    pub size: Point2D,
}

impl MappedKeymap {
    pub fn new(layers: u64, rows: u64, columns: u64) -> Self {
        Self {
            keys: vec![vec![vec![None; columns as usize]; rows as usize]; layers as usize],
            dimensions: Point3D {
                z: layers,
                y: rows,
                x: columns,
            },
            size: Point2D::default(),
        }
    }

    pub fn insert(&mut self, key: KeymapKey, layout: LayoutEntry) {
        let position = key.position;

        self.keys[position.z as usize][position.y as usize][position.x as usize] =
            Some(MappedKeymapKey { key, layout });
        self.size.x = self.size.x.max(position.x);
        self.size.y = self.size.y.max(position.y);
    }
}
