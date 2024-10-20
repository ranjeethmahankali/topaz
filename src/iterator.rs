use crate::mesh::Mesh;

pub struct OutgoingCCWHalfedgeIter<'a> {
    mesh: &'a Mesh,
    hstart: Option<u32>,
    hcurrent: Option<u32>,
}

impl<'a> OutgoingCCWHalfedgeIter<'a> {
    pub fn from(mesh: &'a Mesh, v: u32) -> Self {
        let h = mesh.vertex_halfedge(v);
        OutgoingCCWHalfedgeIter {
            mesh,
            hstart: h,
            hcurrent: h,
        }
    }
}

impl<'a> Iterator for OutgoingCCWHalfedgeIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.hcurrent {
            Some(current) => {
                let next = self
                    .mesh
                    .opposite_halfedge(self.mesh.prev_halfedge(current));
                self.hcurrent = match self.hstart {
                    Some(start) if start != next => Some(next),
                    _ => None,
                };
                Some(current)
            }
            None => None,
        }
    }
}

pub type OutgoingHalfedgeIter<'a> = OutgoingCCWHalfedgeIter<'a>;

pub struct OutgoingCWHalfedgeIter<'a> {
    mesh: &'a Mesh,
    hstart: Option<u32>,
    hcurrent: Option<u32>,
}

impl<'a> OutgoingCWHalfedgeIter<'a> {
    pub fn from(mesh: &'a Mesh, v: u32) -> Self {
        let h = mesh.vertex_halfedge(v);
        OutgoingCWHalfedgeIter {
            mesh,
            hstart: h,
            hcurrent: h,
        }
    }
}

impl<'a> Iterator for OutgoingCWHalfedgeIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        match self.hcurrent {
            Some(current) => {
                let next = self
                    .mesh
                    .next_halfedge(self.mesh.opposite_halfedge(current));
                self.hcurrent = match self.hstart {
                    Some(start) if start != next => Some(next),
                    _ => None,
                };
                Some(current)
            }
            None => None,
        }
    }
}
