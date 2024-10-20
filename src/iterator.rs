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
