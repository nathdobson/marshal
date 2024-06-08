use std::{
    any::Any,
    fmt::{Display, Formatter},
};
use std::any::type_name;

use type_map::TypeMap;

pub struct Context {
    map: TypeMap,
}

impl Context {
    pub fn new() -> Self {
        Context {
            map: TypeMap::new(),
        }
    }
    pub fn insert<T: Any>(&mut self, value: T) {
        self.map.insert(value);
    }
    pub fn get<T: Any>(&self) -> Result<&T, GetError> {
        self.map
            .get::<T>()
            .ok_or_else(|| GetError(type_name::<T>()))
    }
    pub fn get_or_default<T: Any + Default>(&mut self) -> &mut T {
        self.map.entry::<T>().or_insert_with(T::default)
    }
}

#[derive(Debug)]
pub struct GetError(&'static str);

impl Display for GetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find `{}' in DeserializeContext", self.0)
    }
}

impl std::error::Error for GetError {}
