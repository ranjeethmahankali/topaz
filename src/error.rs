pub enum Error {
    // Properties.
    ReadPropertyFailed,
    WriteToPropertyFailed,
    PropertyDoesNotExist,
    // Topology.
    ComplexVertex(u32),
    ComplexEdge(u32),
    HalfedgeNotFound,
}
