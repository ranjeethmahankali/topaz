use crate::{
    error::Error,
    iterator::OutgoingCWHalfedgeIter,
    property::Property,
    topol::{TopolCache, Topology},
};

pub struct Mesh {
    topol: Topology,
    cache: TopolCache,
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
        Mesh {
            topol,
            points,
            cache: TopolCache::default(),
        }
    }

    pub fn with_capacity(nverts: usize, nedges: usize, nfaces: usize) -> Self {
        let mut topol = Topology::with_capacity(nverts, nedges, nfaces);
        let points = topol.create_vertex_prop::<glam::Vec3>();
        Mesh {
            topol,
            points,
            cache: TopolCache::default(),
        }
    }

    pub fn point(&self, vi: u32) -> Result<glam::Vec3, Error> {
        self.points.get(vi)
    }

    pub fn from_vertex(&self, h: u32) -> u32 {
        self.topol.from_vertex(h)
    }

    pub fn halfedge_face(&self, h: u32) -> Option<u32> {
        self.topol.halfedge_face(h)
    }

    pub fn face_halfedge(&self, f: u32) -> u32 {
        self.topol.face_halfedge(f)
    }

    pub fn cw_rotated_halfedge(&self, h: u32) -> u32 {
        self.topol.cw_rotated_halfedge(h)
    }

    pub fn ccw_rotated_halfedge(&self, h: u32) -> u32 {
        self.topol.ccw_rotated_halfedge(h)
    }

    pub fn voh_cw_iter(&self, v: u32) -> OutgoingCWHalfedgeIter {
        OutgoingCWHalfedgeIter::from(&self.topol, v)
    }

    pub fn add_vertex(&mut self, pos: glam::Vec3) -> Result<u32, Error> {
        let vi = self.topol.add_vertex()?;
        self.points.set(vi, pos)?;
        Ok(vi)
    }

    pub fn add_face(&mut self, verts: &[u32]) -> Result<u32, Error> {
        self.topol.add_face(verts, &mut self.cache)
    }

    pub fn add_tri_face(&mut self, v0: u32, v1: u32, v2: u32) -> Result<u32, Error> {
        self.add_face(&[v0, v1, v2])
    }

    pub fn add_quad_face(&mut self, v0: u32, v1: u32, v2: u32, v3: u32) -> Result<u32, Error> {
        self.add_face(&[v0, v1, v2, v3])
    }
}
