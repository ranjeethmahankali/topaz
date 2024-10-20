use std::sync::{Arc, RwLock, Weak};

enum Error {
    ReadPropertyFailed,
    WriteToPropertyFailed,
    PropertyDoesNotExist,
}

struct Vertex {
    halfedge: Option<u32>,
}

struct Halfedge {
    face: Option<u32>,
    vertex: u32,
    next: u32,
    prev: u32,
}

struct Edge {
    halfedges: [Halfedge; 2],
}

struct Face {
    halfedge: u32,
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    faces: Vec<Face>,
    points: Property<glam::Vec3>,
    vprops: PropertyContainer,
}

impl Mesh {
    pub fn new() -> Self {
        let points = Property::<glam::Vec3>::new();
        let mut vprops = PropertyContainer::new();
        vprops.push_property(points.generic_ref());
        Mesh {
            vertices: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            points,
            vprops,
        }
    }

    pub fn with_capacity(nverts: usize, nedges: usize, nfaces: usize) -> Self {
        let points = Property::<glam::Vec3>::with_capacity(nverts);
        let mut vprops = PropertyContainer::new();
        vprops.push_property(points.generic_ref());
        Mesh {
            vertices: Vec::with_capacity(nverts),
            edges: Vec::with_capacity(nedges),
            faces: Vec::with_capacity(nfaces),
            points,
            vprops,
        }
    }

    pub fn halfedge(&self, h: u32) -> &Halfedge {
        &self.edges[(h << 1) as usize].halfedges[(h & 1) as usize]
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

    pub fn add_vertex(&mut self, pos: glam::Vec3) -> Result<u32, Error> {
        let vi = self.vertices.len() as u32;
        self.vprops.push_value()?;
        self.points.set(vi, pos)?;
        return Ok(vi);
    }

    pub fn add_face(&mut self, verts: &[u32]) -> u32 {
        todo!("Not Implemented");
    }

    pub fn add_tri_face(&mut self, v0: u32, v1: u32, v2: u32) -> u32 {
        self.add_face(&[v0, v1, v2])
    }

    pub fn add_quad_face(&mut self, v0: u32, v1: u32, v2: u32, v3: u32) -> u32 {
        self.add_face(&[v0, v1, v2, v3])
    }
}

struct PropertyContainer {
    props: Vec<Box<dyn GenericProperty>>,
}

impl PropertyContainer {
    fn new() -> Self {
        PropertyContainer { props: Vec::new() }
    }

    fn push_property(&mut self, prop: Box<dyn GenericProperty>) {
        self.props.push(prop);
    }

    fn reserve(&mut self, n: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.reserve(n)?;
        }
        return Ok(());
    }

    fn resize(&mut self, n: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.resize(n)?;
        }
        return Ok(());
    }

    fn clear(&mut self) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.clear()?;
        }
        return Ok(());
    }

    fn push_value(&mut self) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.push()?;
        }
        return Ok(());
    }

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.swap(i, j)?;
        }
        return Ok(());
    }

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.copy(src, dst)?;
        }
        return Ok(());
    }

    fn len(&self) -> Result<usize, Error> {
        let first = match self.props.first() {
            Some(first) => first.len()?,
            None => return Ok(0),
        };
        for prop in self.props.iter().skip(1) {
            assert_eq!(first, prop.len()?);
        }
        return Ok(first);
    }
}

// 'static lifetime enforces the data stored inside properties is fully owned
// and doesn't contain any weird references.
trait TPropData: Default + Clone + Copy + 'static {}

impl TPropData for glam::Vec3 {}

trait GenericProperty {
    fn reserve(&mut self, n: usize) -> Result<(), Error>;

    fn resize(&mut self, n: usize) -> Result<(), Error>;

    fn clear(&mut self) -> Result<(), Error>;

    fn push(&mut self) -> Result<(), Error>;

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error>;

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error>;

    fn len(&self) -> Result<usize, Error>;
}

struct Property<T: TPropData> {
    data: Arc<RwLock<Vec<T>>>,
}

impl<T: TPropData> Property<T> {
    fn new() -> Self {
        Property {
            data: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn with_capacity(n: usize) -> Self {
        Property {
            data: Arc::new(RwLock::new(Vec::with_capacity(n))),
        }
    }

    fn generic_ref(&self) -> Box<dyn GenericProperty> {
        Box::new(PropertyRef {
            data: Arc::downgrade(&self.data),
        })
    }

    fn get(&self, i: u32) -> Result<T, Error> {
        self.data
            .read()
            .map_err(|_| Error::ReadPropertyFailed)?
            .get(i as usize)
            .ok_or(Error::ReadPropertyFailed)
            .copied()
    }

    fn set(&mut self, i: u32, val: T) -> Result<(), Error> {
        let mut buf = self
            .data
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?;
        buf[i as usize] = val;
        return Ok(());
    }
}

impl<T: TPropData> Default for Property<T> {
    fn default() -> Self {
        Self {
            data: Default::default(),
        }
    }
}

struct PropertyRef<T: TPropData> {
    data: Weak<RwLock<Vec<T>>>,
}

impl<T: TPropData> PropertyRef<T> {
    fn upgrade(&self) -> Result<Arc<RwLock<Vec<T>>>, Error> {
        self.data.upgrade().ok_or(Error::PropertyDoesNotExist)
    }
}

impl<T: TPropData> GenericProperty for PropertyRef<T> {
    fn reserve(&mut self, n: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .reserve(n); // reserve memory.
        return Ok(());
    }

    fn resize(&mut self, n: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .resize(n, T::default());
        return Ok(());
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .clear();
        return Ok(());
    }

    fn push(&mut self) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .push(T::default());
        return Ok(());
    }

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .swap(i, j);
        return Ok(());
    }

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .copy_within(src..(src + 1), dst);
        return Ok(());
    }

    fn len(&self) -> Result<usize, Error> {
        Ok(self
            .upgrade()?
            .read()
            .map_err(|_| Error::ReadPropertyFailed)?
            .len())
    }
}
