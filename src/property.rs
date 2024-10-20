use std::sync::{Arc, RwLock, Weak};

use crate::error::Error;

pub struct PropertyContainer {
    props: Vec<Box<dyn GenericProperty>>,
}

impl Default for PropertyContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl PropertyContainer {
    pub fn new() -> Self {
        PropertyContainer { props: Vec::new() }
    }

    fn push_property(&mut self, prop: Box<dyn GenericProperty>) {
        self.props.push(prop);
    }

    pub fn reserve(&mut self, n: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.reserve(n)?;
        }
        Ok(())
    }

    pub fn resize(&mut self, n: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.resize(n)?;
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.clear()?;
        }
        Ok(())
    }

    pub fn push_value(&mut self) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.push()?;
        }
        Ok(())
    }

    pub fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.swap(i, j)?;
        }
        Ok(())
    }

    pub fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error> {
        for prop in self.props.iter_mut() {
            prop.copy(src, dst)?;
        }
        Ok(())
    }

    pub fn len(&self) -> Result<usize, Error> {
        let first = match self.props.first() {
            Some(first) => first.len()?,
            None => return Ok(0),
        };
        for prop in self.props.iter().skip(1) {
            assert_eq!(first, prop.len()?);
        }
        Ok(first)
    }
}

// 'static lifetime enforces the data stored inside properties is fully owned
// and doesn't contain any weird references.
pub trait TPropData: Default + Clone + Copy + 'static {}

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

pub struct Property<T: TPropData> {
    data: Arc<RwLock<Vec<T>>>,
}

impl<T: TPropData> Property<T> {
    pub fn new(container: &mut PropertyContainer) -> Self {
        let prop = Property {
            data: Arc::new(RwLock::new(Vec::new())),
        };
        container.push_property(prop.generic_ref());
        prop
    }

    pub fn with_capacity(n: usize, container: &mut PropertyContainer) -> Self {
        let prop = Property {
            data: Arc::new(RwLock::new(Vec::with_capacity(n))),
        };
        container.push_property(prop.generic_ref());
        prop
    }

    fn generic_ref(&self) -> Box<dyn GenericProperty> {
        Box::new(PropertyRef {
            data: Arc::downgrade(&self.data),
        })
    }

    pub fn get(&self, i: u32) -> Result<T, Error> {
        self.data
            .read()
            .map_err(|_| Error::ReadPropertyFailed)?
            .get(i as usize)
            .ok_or(Error::ReadPropertyFailed)
            .copied()
    }

    pub fn set(&mut self, i: u32, val: T) -> Result<(), Error> {
        let mut buf = self
            .data
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?;
        buf[i as usize] = val;
        Ok(())
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
        Ok(())
    }

    fn resize(&mut self, n: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .resize(n, T::default());
        Ok(())
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .clear();
        Ok(())
    }

    fn push(&mut self) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .push(T::default());
        Ok(())
    }

    fn swap(&mut self, i: usize, j: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .swap(i, j);
        Ok(())
    }

    fn copy(&mut self, src: usize, dst: usize) -> Result<(), Error> {
        self.upgrade()?
            .write()
            .map_err(|_| Error::WriteToPropertyFailed)?
            .copy_within(src..(src + 1), dst);
        Ok(())
    }

    fn len(&self) -> Result<usize, Error> {
        Ok(self
            .upgrade()?
            .read()
            .map_err(|_| Error::ReadPropertyFailed)?
            .len())
    }
}
