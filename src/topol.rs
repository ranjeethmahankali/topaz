use crate::{
    error::Error,
    iterator::OutgoingHalfedgeIter,
    property::{Property, PropertyContainer, TPropData},
};

enum TentativeEdge {
    Old(u32),
    New {
        index: u32,
        from: u32,
        to: u32,
        prev: Option<u32>,
        next: Option<u32>,
        opp_prev: Option<u32>,
        opp_next: Option<u32>,
    },
}

#[derive(Default)]
pub(crate) struct TopolCache {
    halfedges: Vec<Option<u32>>,
    needs_adjust: Vec<bool>,
    next_cache: Vec<(u32, u32)>,
    tentative: Vec<TentativeEdge>,
}

pub(crate) struct Vertex {
    halfedge: Option<u32>,
}

pub(crate) struct Halfedge {
    face: Option<u32>,
    vertex: u32,
    next: u32,
    prev: u32,
}

pub(crate) struct Edge {
    halfedges: [Halfedge; 2],
}

pub(crate) struct Face {
    halfedge: u32,
}

pub(crate) struct Topology {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    vprops: PropertyContainer,
    fprops: PropertyContainer,
}

impl Topology {
    pub fn new() -> Self {
        Topology {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            vprops: PropertyContainer::new(),
            fprops: PropertyContainer::new(),
        }
    }

    pub fn with_capacity(nverts: usize, nedges: usize, nfaces: usize) -> Self {
        Topology {
            vertices: Vec::with_capacity(nverts),
            edges: Vec::with_capacity(nedges),
            faces: Vec::with_capacity(nfaces),
            vprops: PropertyContainer::new(),
            fprops: PropertyContainer::new(),
        }
    }

    pub fn create_vertex_prop<T: TPropData>(&mut self) -> Property<T> {
        Property::<T>::new(&mut self.vprops)
    }

    fn vertex(&self, v: u32) -> &Vertex {
        &self.vertices[v as usize]
    }

    fn halfedge(&self, h: u32) -> &Halfedge {
        &self.edges[(h >> 1) as usize].halfedges[(h & 1) as usize]
    }

    fn halfedge_mut(&mut self, h: u32) -> &mut Halfedge {
        &mut self.edges[(h >> 1) as usize].halfedges[(h & 1) as usize]
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
        self.faces[f as usize].halfedge
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

    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    pub fn num_edges(&self) -> usize {
        self.edges.len()
    }

    pub fn num_halfedges(&self) -> usize {
        self.num_edges() * 2
    }

    pub fn num_faces(&self) -> usize {
        self.faces.len()
    }

    pub fn find_halfedge(&self, from: u32, to: u32) -> Option<u32> {
        OutgoingHalfedgeIter::from(self, from).find(|h| self.to_vertex(*h) == to)
    }

    pub fn add_vertex(&mut self) -> Result<u32, Error> {
        let vi = self.vertices.len() as u32;
        self.vprops.push_value()?;
        Ok(vi)
    }

    fn new_face(&mut self, halfedge: u32) -> Result<u32, Error> {
        let fi = self.faces.len() as u32;
        self.fprops.push_value()?;
        self.faces.push(Face { halfedge });
        Ok(fi)
    }

    pub fn set_vertex_halfedge(&mut self, v: u32, h: u32) {
        self.vertices[v as usize].halfedge = Some(h);
    }

    pub fn set_next_halfedge(&mut self, hprev: u32, hnext: u32) {
        self.halfedge_mut(hprev).next = hnext;
        self.halfedge_mut(hnext).prev = hprev;
    }

    fn adjust_outgoing_halfedge(&mut self, v: u32) {
        match OutgoingHalfedgeIter::from(self, v).find(|h| self.is_boundary_halfedge(*h)) {
            Some(h) => self.set_vertex_halfedge(v, h),
            None => {} // Do nothing.
        }
    }

    fn new_edge(
        &mut self,
        from: u32,
        to: u32,
        prev: u32,
        next: u32,
        opp_prev: u32,
        opp_next: u32,
    ) -> u32 {
        let ei = self.edges.len() as u32;
        self.edges.push(Edge {
            halfedges: [
                Halfedge {
                    face: None,
                    vertex: to,
                    next,
                    prev,
                },
                Halfedge {
                    face: None,
                    vertex: from,
                    next: opp_next,
                    prev: opp_prev,
                },
            ],
        });
        ei
    }

    pub fn add_face(&mut self, verts: &[u32], cache: &mut TopolCache) -> Result<u32, Error> {
        cache.halfedges.reserve(verts.len());
        cache.needs_adjust.reserve(verts.len());
        cache.next_cache.reserve(verts.len() * 6);
        // Check for topological errors.
        for i in 0..verts.len() {
            if self.is_boundary_vertex(verts[i]) {
                // Ensure vertex is manifold.
                return Err(Error::ComplexVertex(verts[i]));
            }
            // Ensure edge is manifold.
            let h = self.find_halfedge(verts[i], verts[(i + 1) % verts.len()]);
            match h {
                Some(h) if !self.is_boundary_halfedge(h) => return Err(Error::ComplexEdge(h)),
                _ => {} // Do nothing.
            }
            cache.halfedges.push(h);
            cache.needs_adjust.push(false);
        }
        // If any vertex has more than two incident boundary edges, relinking might be necessary.
        for (prev, next) in (0..verts.len()).filter_map(|i| {
            match (cache.halfedges[i], cache.halfedges[(i + 1) % verts.len()]) {
                (Some(prev), Some(next)) if self.next_halfedge(prev) != next => Some((prev, next)),
                _ => None,
            }
        }) {
            // Relink the patch.
            let boundprev = {
                let mut out = self.opposite_halfedge(next);
                loop {
                    out = self.opposite_halfedge(self.next_halfedge(out));
                    if self.is_boundary_halfedge(out) {
                        break;
                    }
                }
                out
            };
            let boundnext = self.next_halfedge(boundprev);
            // Ok ?
            if boundprev == prev {
                return Err(Error::PatchRelinkingFailed);
            }
            debug_assert!(
                self.is_boundary_halfedge(boundprev) && self.is_boundary_halfedge(boundnext)
            );
            // other halfedges.
            let pstart = self.next_halfedge(prev);
            let pend = self.prev_halfedge(next);
            // relink.
            cache.next_cache.extend_from_slice(&[
                (boundprev, pstart),
                (pend, boundnext),
                (prev, next),
            ]);
        }
        // Create boundary loop. No more errors allowed from this point.
        // If anything goes wrong, we panic.
        cache.tentative.clear();
        cache.tentative.reserve(verts.len());
        {
            let mut ei = self.edges.len() as u32;
            cache
                .tentative
                .extend((0..verts.len()).map(|i| match cache.halfedges[i] {
                    Some(h) => TentativeEdge::Old(h),
                    None => TentativeEdge::New {
                        index: {
                            let current = ei;
                            ei += 1;
                            current << 1
                        },
                        from: verts[i],
                        to: verts[(i + 1) % verts.len()],
                        prev: None,
                        next: None,
                        opp_prev: None,
                        opp_next: None,
                    },
                }));
        }
        for (i, j) in (0..verts.len()).map(|i| (i, (i + 1) % verts.len())) {
            let (e0, e1) = if j == 0 {
                let (right, left) = cache.tentative.split_at_mut(i);
                (&mut left[0], &mut right[0])
            } else {
                let (left, right) = cache.tentative.split_at_mut(j);
                (&mut left[left.len() - 1], &mut right[0])
            };
            let v = verts[j];
            match (e0, e1) {
                (TentativeEdge::Old(_), TentativeEdge::Old(innernext)) => {
                    cache.needs_adjust[j] = self.vertex_halfedge(v) == Some(*innernext);
                }
                (
                    TentativeEdge::New {
                        index: innerprev,
                        opp_prev,
                        next,
                        ..
                    },
                    TentativeEdge::Old(innernext),
                ) => {
                    let innernext = *innernext;
                    let innerprev = *innerprev;
                    let outernext = self.opposite_halfedge(innerprev);
                    let boundprev = self.prev_halfedge(innernext);
                    cache.next_cache.push((boundprev, outernext));
                    *opp_prev = Some(boundprev);
                    cache.next_cache.push((innerprev, innernext));
                    *next = Some(innernext);
                    self.set_vertex_halfedge(v, outernext);
                }
                (
                    TentativeEdge::Old(innerprev),
                    TentativeEdge::New {
                        index: innernext,
                        prev,
                        opp_next,
                        ..
                    },
                ) => {
                    let innerprev = *innerprev;
                    let innernext = *innernext;
                    let outerprev = self.opposite_halfedge(innernext);
                    let boundnext = self.next_halfedge(innerprev);
                    cache.next_cache.push((outerprev, boundnext));
                    *opp_next = Some(boundnext);
                    cache.next_cache.push((innerprev, innernext));
                    *prev = Some(innerprev);
                    self.set_vertex_halfedge(v, boundnext);
                }
                (
                    TentativeEdge::New {
                        index: innerprev,
                        next,
                        opp_prev,
                        ..
                    },
                    TentativeEdge::New {
                        index: innernext,
                        prev,
                        opp_next,
                        ..
                    },
                ) => {
                    let innerprev = *innerprev;
                    let innernext = *innernext;
                    let outernext = self.opposite_halfedge(innerprev);
                    let outerprev = self.opposite_halfedge(innernext);
                    if let Some(boundnext) = self.vertex_halfedge(v) {
                        let boundprev = self.prev_halfedge(boundnext);
                        cache
                            .next_cache
                            .extend(&[(boundprev, outernext), (outerprev, boundnext)]);
                        *next = Some(innernext);
                        *opp_prev = Some(boundprev);
                        *prev = Some(innerprev);
                        *opp_next = Some(boundnext);
                    } else {
                        self.set_vertex_halfedge(v, outernext);
                        *next = Some(innernext);
                        *opp_prev = Some(outerprev);
                        *prev = Some(innerprev);
                        *opp_next = Some(outernext);
                    }
                }
            };
        }
        // Convert tentative edges into real edges.
        const ERR: &str = "Unable to create edge loop";
        for tedge in &cache.tentative {
            match tedge {
                TentativeEdge::Old(_) => continue,
                TentativeEdge::New {
                    index,
                    from,
                    to,
                    prev,
                    next,
                    opp_prev,
                    opp_next,
                } => {
                    let ei = self.new_edge(
                        *from,
                        *to,
                        prev.expect(ERR),
                        next.expect(ERR),
                        opp_prev.expect(ERR),
                        opp_next.expect(ERR),
                    );
                    assert!(*index == ei, "Failed to create an edge loop");
                }
            }
        }
        // Create the face.
        let fnew = self.new_face(match cache.tentative.last().expect(ERR) {
            TentativeEdge::Old(index) => *index,
            TentativeEdge::New { index, .. } => *index,
        })?;
        // Process next halfedge cache.
        for (prev, next) in cache.next_cache.drain(..) {
            self.set_next_halfedge(prev, next);
        }
        // Adjust vertices' halfedge handles.
        for i in 0..verts.len() {
            if cache.needs_adjust[i] {
                self.adjust_outgoing_halfedge(verts[i]);
            }
        }
        Ok(fnew)
    }
}

impl Default for Topology {
    fn default() -> Self {
        Self::new()
    }
}
