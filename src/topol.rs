use crate::{
    error::Error,
    iterator::OutgoingHalfedgeIter,
    property::{Property, PropertyContainer, TPropData},
};

struct EdgeCache {
    halfedge: Option<u32>,
    needs_adjust: bool,
}

#[derive(Default)]
struct Cache {
    edge_data: Vec<EdgeCache>,
}

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

pub struct Topology {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    vprops: PropertyContainer,
    fprops: PropertyContainer,
    cache: Cache,
}

impl Topology {
    pub fn new() -> Self {
        Topology {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            vprops: PropertyContainer::new(),
            fprops: PropertyContainer::new(),
            cache: Cache::default(),
        }
    }

    pub fn with_capacity(nverts: usize, nedges: usize, nfaces: usize) -> Self {
        Topology {
            vertices: Vec::with_capacity(nverts),
            edges: Vec::with_capacity(nedges),
            faces: Vec::with_capacity(nfaces),
            vprops: PropertyContainer::new(),
            fprops: PropertyContainer::new(),
            cache: Cache::default(),
        }
    }

    pub fn create_vertex_prop<T: TPropData>(&mut self) -> Property<T> {
        Property::<T>::new(&mut self.vprops)
    }

    fn vertex(&self, v: u32) -> &Vertex {
        &self.vertices[v as usize]
    }

    fn halfedge(&self, h: u32) -> &Halfedge {
        &self.edge(h >> 1).halfedges[(h & 1) as usize]
    }

    fn edge(&self, e: u32) -> &Edge {
        &self.edges[e as usize]
    }

    fn face(&self, f: u32) -> &Face {
        &self.faces[f as usize]
    }

    fn face_mut(&mut self, f: u32) -> &mut Face {
        &mut self.faces[f as usize]
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

    pub fn add_vertex(&mut self) -> Result<u32, Error> {
        let vi = self.vertices.len() as u32;
        self.vprops.push_value()?;
        Ok(vi)
    }

    fn new_face(&mut self) -> Result<u32, Error> {
        let fi = self.faces.len() as u32;
        self.fprops.push_value()?;
        Ok(fi)
    }

    fn adjust_outgoing_halfedge(&mut self, _v: u32) {
        todo!("Not Implemented");
    }

    pub fn set_face_halfedge(&mut self, f: u32, h: u32) {
        self.face_mut(f).halfedge = h;
    }

    pub fn add_face(&mut self, verts: &[u32]) -> Result<u32, Error> {
        // Check for topological errors.
        for i in 0..verts.len() {
            if self.is_boundary_vertex(verts[i]) {
                // Ensure vertex is manifold.
                return Err(Error::ComplexVertex(verts[i]));
            }
            let j = (i + 1) % verts.len();
            // Ensure edge is manifold.
            let h = self.find_halfedge(verts[i], verts[j]);
            match h {
                Some(h) if !self.is_boundary_halfedge(h) => return Err(Error::ComplexEdge(h)),
                _ => {} // Do nothing.
            }
            self.cache.edge_data.push(EdgeCache {
                halfedge: h,
                needs_adjust: false,
            });
        }
        // Find consecutive halfedge pairs that need relinking, and relink the patches.
        for (_prev, _next) in (0..verts.len()).filter_map(|i| {
            let j = (i + 1) % verts.len();
            match (
                self.cache.edge_data[i].halfedge,
                self.cache.edge_data[j].halfedge,
            ) {
                (Some(prev), Some(next)) if self.next_halfedge(prev) != next => Some((prev, next)),
                _ => None,
            }
        }) {
            // Relink the whole patch.
            todo!("Not Implemented");
        }
        // Create missing edges.
        for (_i, _j) in (0..verts.len()).filter_map(|i| {
            if self.cache.edge_data[i].halfedge.is_none() {
                Some((i, (i + 1) % verts.len()))
            } else {
                None
            }
        }) {
            todo!("Not Implemented");
        }
        // Create the face.
        let fnew = self.new_face()?;
        self.set_face_halfedge(
            fnew,
            self.cache
                .edge_data
                .last()
                .ok_or(Error::HalfedgeNotFound)?
                .halfedge
                .ok_or(Error::HalfedgeNotFound)?,
        );
        // Setup halfedges.
        for _i in 0..verts.len() {
            todo!("Not Implemented");
        }
        // Process next halfedge cache.
        for _i in 0..verts.len() {
            todo!("Not Implemented");
        }
        // Adjust vertices' halfedge handles.
        for i in 0..verts.len() {
            if self.cache.edge_data[i].needs_adjust {
                self.adjust_outgoing_halfedge(verts[i]);
            }
        }
        Ok(fnew)
    }

    pub fn add_tri_face(&mut self, v0: u32, v1: u32, v2: u32) -> Result<u32, Error> {
        self.add_face(&[v0, v1, v2])
    }

    pub fn add_quad_face(&mut self, v0: u32, v1: u32, v2: u32, v3: u32) -> Result<u32, Error> {
        self.add_face(&[v0, v1, v2, v3])
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}
