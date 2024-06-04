use std::{
    any::Any,
    fmt::{Display, Formatter},
};
use std::any::type_name;

use type_map::TypeMap;

pub struct DeserializeContext {
    map: TypeMap,
}

impl DeserializeContext {
    pub fn new() -> Self {
        DeserializeContext {
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
}

pub struct GetError(&'static str);

impl Display for GetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find {} in DeserializeContext", self.0)
    }
}
