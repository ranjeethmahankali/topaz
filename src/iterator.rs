use crate::topol::Topology;

pub struct OutgoingCCWHalfedgeIter<'a> {
    topol: &'a Topology,
    hstart: Option<u32>,
    hcurrent: Option<u32>,
}

impl<'a> OutgoingCCWHalfedgeIter<'a> {
    pub(crate) fn from(topol: &'a Topology, v: u32) -> Self {
        let h = topol.vertex_halfedge(v);
        OutgoingCCWHalfedgeIter {
            topol,
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
                    .topol
                    .opposite_halfedge(self.topol.prev_halfedge(current));
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
    topol: &'a Topology,
    hstart: Option<u32>,
    hcurrent: Option<u32>,
}

impl<'a> OutgoingCWHalfedgeIter<'a> {
    pub(crate) fn from(topol: &'a Topology, v: u32) -> Self {
        let h = topol.vertex_halfedge(v);
        OutgoingCWHalfedgeIter {
            topol,
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
                    .topol
                    .next_halfedge(self.topol.opposite_halfedge(current));
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

pub struct VertexCCWFaceIter<'a> {
    outgoing: OutgoingCCWHalfedgeIter<'a>,
}

impl<'a> VertexCCWFaceIter<'a> {
    pub(crate) fn from(topol: &'a Topology, v: u32) -> Self {
        VertexCCWFaceIter {
            outgoing: OutgoingCCWHalfedgeIter::from(topol, v),
        }
    }
}

impl<'a> Iterator for VertexCCWFaceIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.outgoing.topol.halfedge_face(self.outgoing.next()?)
    }
}

pub struct VertexCWFaceIter<'a> {
    outgoing: OutgoingCWHalfedgeIter<'a>,
}

impl<'a> VertexCWFaceIter<'a> {
    pub(crate) fn from(topol: &'a Topology, v: u32) -> Self {
        VertexCWFaceIter {
            outgoing: OutgoingCWHalfedgeIter::from(topol, v),
        }
    }
}

impl<'a> Iterator for VertexCWFaceIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        self.outgoing.topol.halfedge_face(self.outgoing.next()?)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        iterator::{VertexCCWFaceIter, VertexCWFaceIter},
        topol::{TopolCache, Topology},
    };

    fn quad_box() -> Topology {
        let mut topol = Topology::with_capacity(8, 12, 6);
        let verts: Vec<_> = (0..8)
            .map(|_| topol.add_vertex().expect("Unable to add a vertex"))
            .collect();
        assert_eq!(verts, (0u32..8).collect::<Vec<_>>());
        let mut cache = TopolCache::default();
        let faces: Vec<_> = [
            [0u32, 3, 2, 1],
            [0, 1, 5, 4],
            [1, 2, 6, 5],
            [2, 3, 7, 6],
            [3, 0, 4, 7],
            [4, 5, 6, 7],
        ]
        .iter()
        .map(|indices| {
            topol
                .add_face(indices, &mut cache)
                .expect("Unable to add a face")
        })
        .collect();
        assert_eq!(faces, (0..6).collect::<Vec<_>>());
        assert_eq!(topol.num_vertices(), 8);
        assert_eq!(topol.num_halfedges(), 24);
        assert_eq!(topol.num_edges(), 12);
        assert_eq!(topol.num_faces(), 6);
        topol
    }

    #[test]
    fn t_box_vertex_ccw_faces() {
        let qbox = quad_box();
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 0).collect::<Vec<_>>(),
            vec![4, 0, 1]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 1).collect::<Vec<_>>(),
            vec![2, 1, 0]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 2).collect::<Vec<_>>(),
            vec![3, 2, 0]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 3).collect::<Vec<_>>(),
            vec![4, 3, 0]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 4).collect::<Vec<_>>(),
            vec![5, 4, 1]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 5).collect::<Vec<_>>(),
            vec![5, 1, 2]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 6).collect::<Vec<_>>(),
            vec![5, 2, 3]
        );
        assert_eq!(
            VertexCCWFaceIter::from(&qbox, 7).collect::<Vec<_>>(),
            vec![5, 3, 4]
        );
    }

    #[test]
    fn t_box_vertex_cw_faces() {
        let qbox = quad_box();
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 0).collect::<Vec<_>>(),
            vec![4, 1, 0]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 1).collect::<Vec<_>>(),
            vec![2, 0, 1]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 2).collect::<Vec<_>>(),
            vec![3, 0, 2]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 3).collect::<Vec<_>>(),
            vec![4, 0, 3]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 4).collect::<Vec<_>>(),
            vec![5, 1, 4]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 5).collect::<Vec<_>>(),
            vec![5, 2, 1]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 6).collect::<Vec<_>>(),
            vec![5, 3, 2]
        );
        assert_eq!(
            VertexCWFaceIter::from(&qbox, 7).collect::<Vec<_>>(),
            vec![5, 4, 3]
        );
    }
}
