pub enum Error {
    // Properties.
    BorrowedPropertyAccess,
    PropertyDoesNotExist,
    // Topology.
    ComplexVertex(u32),
    ComplexEdge(u32),
    HalfedgeNotFound,
}
