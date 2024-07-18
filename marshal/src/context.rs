use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::fmt::Debug;
use std::{
    any::Any,
    fmt::{Display, Formatter},
};

trait ContextEntry: Any {
    fn type_name_dyn(&self) -> &'static str;
}

impl<T: 'static> ContextEntry for T {
    fn type_name_dyn(&self) -> &'static str {
        type_name::<T>()
    }
}

pub struct MutContext<'ctx> {
    map: HashMap<TypeId, &'ctx mut dyn ContextEntry>,
}

pub struct ConstContext<'ctx> {
    map: HashMap<TypeId, &'ctx dyn ContextEntry>,
}

pub struct OwnedContext<'ctx> {
    mut_ctx: MutContext<'ctx>,
    const_ctx: ConstContext<'ctx>,
}

pub struct Context<'map, 'ctx> {
    mut_ctx: &'map mut MutContext<'ctx>,
    const_ctx: &'map ConstContext<'ctx>,
}

impl<'ctx> MutContext<'ctx> {
    pub fn new() -> Self {
        MutContext {
            map: HashMap::new(),
        }
    }
}

impl<'ctx> ConstContext<'ctx> {
    pub fn new() -> Self {
        ConstContext {
            map: HashMap::new(),
        }
    }
}

impl<'ctx> OwnedContext<'ctx> {
    pub fn new() -> Self {
        OwnedContext {
            mut_ctx: MutContext::new(),
            const_ctx: ConstContext::new(),
        }
    }
    pub fn insert_mut<T: Any>(&mut self, value: &'ctx mut T) {
        log::info!("insert mut {}", type_name::<T>());
        self.mut_ctx.map.insert(TypeId::of::<T>(), value);
    }
    pub fn insert_const<T: Any>(&mut self, value: &'ctx T) {
        log::info!("insert const {}", type_name::<T>());
        self.const_ctx.map.insert(TypeId::of::<T>(), value);
    }
    pub fn borrow<'borrow>(&'borrow mut self) -> Context<'borrow, 'ctx> {
        Context {
            mut_ctx: &mut self.mut_ctx,
            const_ctx: &self.const_ctx,
        }
    }
}

impl<'ctx> ConstContext<'ctx> {
    pub fn get_const<T: Any>(&self) -> Result<&T, GetError> {
        Ok(((*self
            .map
            .get(&TypeId::of::<T>())
            .ok_or_else(|| GetError(type_name::<T>()))?) as &dyn Any)
            .downcast_ref()
            .unwrap())
    }
}

impl<'map, 'ctx> Context<'map, 'ctx> {
    pub fn get_const<T: Any>(&self) -> Result<&T, GetError> {
        self.const_ctx.get_const()
    }
    pub fn get_const_reborrow<'map2, T: Any>(
        &'map2 mut self,
    ) -> Result<(&'map2 T, Context<'map2, 'ctx>), GetError> {
        Ok((
            self.const_ctx.get_const()?,
            Context {
                mut_ctx: &mut *self.mut_ctx,
                const_ctx: &*self.const_ctx,
            },
        ))
    }
    pub fn get_mut<T: Any>(self) -> Result<&'map mut T, GetError> {
        Ok(((*self
            .mut_ctx
            .map
            .get_mut(&TypeId::of::<T>())
            .ok_or_else(|| GetError(type_name::<T>()))?) as &mut dyn Any)
            .downcast_mut()
            .unwrap())
    }
    pub fn clone_scoped(&mut self) -> OwnedContext {
        let mut_map = self
            .mut_ctx
            .map
            .iter_mut()
            .map(|(k, v)| (*k, &mut **v))
            .collect();
        let const_map = self.const_ctx.map.iter().map(|(k, v)| (*k, &**v)).collect();
        OwnedContext {
            mut_ctx: MutContext { map: mut_map },
            const_ctx: ConstContext { map: const_map },
        }
    }
    pub fn insert_mut_scoped<'scope, T: Any>(
        &'scope mut self,
        value: &'scope mut T,
    ) -> OwnedContext<'scope> {
        let mut clone = self.clone_scoped();
        clone.insert_mut(value);
        clone
    }
    pub fn insert_const_scoped<'scope, T: Any>(
        &'scope mut self,
        value: &'scope T,
    ) -> OwnedContext<'scope> {
        let mut clone = self.clone_scoped();
        clone.insert_const(value);
        clone
    }
    #[inline]
    pub fn reborrow<'map2>(&'map2 mut self) -> Context<'map2, 'ctx> {
        Context {
            mut_ctx: self.mut_ctx,
            const_ctx: self.const_ctx,
        }
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

impl<'map, 'ctx> Debug for Context<'map, 'ctx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_struct("Context");
        f.field("const", &self.const_ctx);
        f.field("mut", &self.mut_ctx);
        f.finish()
    }
}

impl<'ctx> Debug for ConstContext<'ctx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_list();
        for x in self.map.values() {
            f.entry(&(**x).type_name_dyn());
        }
        f.finish()
    }
}

impl<'ctx> Debug for MutContext<'ctx> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut f = f.debug_list();
        for x in self.map.values() {
            f.entry(&(**x).type_name_dyn());
        }
        f.finish()
    }
}
