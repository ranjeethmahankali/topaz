use crate::{error::Error, property::Property, topol::Topology};

pub struct Mesh {
    topol: Topology,
    points: Property<glam::Vec3>,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    pub fn new() -> Self {
        let mut topol = Topology::new();
        let points = topol.create_vertex_prop::<glam::Vec3>();
        Mesh { topol, points }
    }

    pub fn with_capacity(nverts: usize, nedges: usize, nfaces: usize) -> Self {
        let mut topol = Topology::with_capacity(nverts, nedges, nfaces);
        let points = topol.create_vertex_prop::<glam::Vec3>();
        Mesh { topol, points }
    }

    pub fn topol(&self) -> &Topology {
        &self.topol
    }

    pub fn add_vertex(&mut self, pos: glam::Vec3) -> Result<u32, Error> {
        let vi = self.topol.add_vertex()?;
        self.points.set(vi, pos)?;
        Ok(vi)
    }

    pub fn point(&self, vi: u32) -> Result<glam::Vec3, Error> {
        self.points.get(vi)
    }
}
