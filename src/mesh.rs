use crate::{
    error::Error,
    iterator::OutgoingHalfedgeIter,
    property::{Property, PropertyContainer},
};

pub struct Vertex {
    halfedge: Option<u32>,
}

pub struct Halfedge {
    face: Option<u32>,
    vertex: u32,
    next: u32,
    prev: u32,
}

pub struct Edge {
    halfedges: [Halfedge; 2],
}

pub struct Face {
    halfedge: u32,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    points: Property<glam::Vec3>,
    vprops: PropertyContainer,
}

impl Default for Mesh {
    fn default() -> Self {
        Self::new()
    }
}

impl Mesh {
    pub fn new() -> Self {
        let mut vprops = PropertyContainer::new();
        let points = Property::<glam::Vec3>::new(&mut vprops);
        Mesh {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            points,
            vprops,
        }
    }

    pub fn with_capacity(nverts: usize, nedges: usize, nfaces: usize) -> Self {
        let mut vprops = PropertyContainer::new();
        let points = Property::<glam::Vec3>::with_capacity(nverts, &mut vprops);
        Mesh {
            vertices: Vec::with_capacity(nverts),
            edges: Vec::with_capacity(nedges),
            faces: Vec::with_capacity(nfaces),
            points,
            vprops,
        }
    }

    pub fn vertex(&self, v: u32) -> &Vertex {
        &self.vertices[v as usize]
    }

    pub fn halfedge(&self, h: u32) -> &Halfedge {
        &self.edge(h >> 1).halfedges[(h & 1) as usize]
    }

    pub fn edge(&self, e: u32) -> &Edge {
        &self.edges[e as usize]
    }

    pub fn face(&self, f: u32) -> &Face {
        &self.faces[f as usize]
    }

    pub fn to_vertex(&self, h: u32) -> u32 {
        self.halfedge(h).vertex
    }

    pub fn from_vertex(&self, h: u32) -> u32 {
        self.halfedge(self.opposite_halfedge(h)).vertex
    }

    pub fn prev_halfedge(&self, h: u32) -> u32 {
        self.halfedge(h).prev
    }

    pub fn next_halfedge(&self, h: u32) -> u32 {
        self.halfedge(h).next
    }

    pub fn halfedge_face(&self, h: u32) -> Option<u32> {
        self.halfedge(h).face
    }

    pub fn face_halfedge(&self, f: u32) -> u32 {
        self.face(f).halfedge
    }

    pub fn vertex_halfedge(&self, v: u32) -> Option<u32> {
        self.vertex(v).halfedge
    }

    pub fn is_boundary_halfedge(&self, h: u32) -> bool {
        self.halfedge(h).face.is_none()
    }

    pub fn is_boundary_vertex(&self, v: u32) -> bool {
        match self.vertices[v as usize].halfedge {
            Some(h) => self.is_boundary_halfedge(h),
            None => true,
        }
    }

    pub const fn opposite_halfedge(&self, h: u32) -> u32 {
        h ^ 1
    }

    pub fn cw_rotated_halfedge(&self, h: u32) -> u32 {
        self.halfedge(self.opposite_halfedge(h)).next
    }

    pub fn ccw_rotated_halfedge(&self, h: u32) -> u32 {
        self.opposite_halfedge(self.halfedge(h).prev)
    }

    pub fn find_halfedge(&self, from: u32, to: u32) -> Option<u32> {
        OutgoingHalfedgeIter::from(self, from).find(|h| self.to_vertex(*h) == to)
    }

    pub fn add_vertex(&mut self, pos: glam::Vec3) -> Result<u32, Error> {
        let vi = self.vertices.len() as u32;
        self.vprops.push_value()?;
        self.points.set(vi, pos)?;
        Ok(vi)
    }

    pub fn add_face(&mut self, verts: &[u32]) -> Result<u32, Error> {
        for i in 0..verts.len() {
            if self.is_boundary_vertex(verts[i]) {
                return Err(Error::ComplexVertex);
            }
            // let j = (i + 1) % verts.len();
        }
        todo!("Not Implemented");
    }

    pub fn add_tri_face(&mut self, v0: u32, v1: u32, v2: u32) -> Result<u32, Error> {
        self.add_face(&[v0, v1, v2])
    }

    pub fn add_quad_face(&mut self, v0: u32, v1: u32, v2: u32, v3: u32) -> Result<u32, Error> {
        self.add_face(&[v0, v1, v2, v3])
    }
}
