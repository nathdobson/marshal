use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::{
    any::Any,
    fmt::{Display, Formatter},
};

pub struct Context<'ctx> {
    map: HashMap<TypeId, &'ctx mut dyn Any>,
}

impl<'ctx> Context<'ctx> {
    pub fn new() -> Self {
        Context {
            map: HashMap::new(),
        }
    }
    pub fn insert<T: Any>(&mut self, value: &'ctx mut T) {
        self.map.insert(TypeId::of::<T>(), value);
    }
    pub fn get<T: Any>(&self) -> Result<&T, GetError> {
        Ok(self
            .map
            .get(&TypeId::of::<T>())
            .ok_or_else(|| GetError(type_name::<T>()))?
            .downcast_ref()
            .unwrap())
    }
    pub fn get_mut<T: Any>(&mut self) -> Result<&mut T, GetError> {
        Ok(self
            .map
            .get_mut(&TypeId::of::<T>())
            .ok_or_else(|| GetError(type_name::<T>()))?
            .downcast_mut()
            .unwrap())
    }
    pub fn insert_scoped<'scope, T: Any>(
        &'scope mut self,
        value: &'scope mut T,
    ) -> Context<'scope> {
        let mut new = Context::new();
        new.insert(value);
        for (k, v) in self.map.iter_mut() {
            new.map.insert(*k, &mut **v);
        }
        new
    }
}

#[derive(Debug)]
pub struct GetError(&'static str);

impl Display for GetError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not find `{}' in Context", self.0)
    }
}

impl std::error::Error for GetError {}
