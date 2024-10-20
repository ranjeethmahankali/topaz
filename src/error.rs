pub enum Error {
    // Properties.
    BorrowedPropertyAccess,
    PropertyDoesNotExist,
    OutOfBoundsAccess,
    // Topology.
    ComplexVertex(u32),
    ComplexEdge(u32),
    HalfedgeNotFound,
    PatchRelinkingFailed,
}
