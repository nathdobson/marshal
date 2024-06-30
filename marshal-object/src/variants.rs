use crate::{Object, OBJECT_REGISTRY};
use std::marker::PhantomData;
use std::ops::Index;
use type_map::concurrent::TypeMap;

pub struct VariantImplTable<O: Object, DV: VariantImpl> {
    variants: Vec<&'static DV>,
    phantom: PhantomData<O>,
}

pub trait VariantImpl: 'static + Sync + Send {}

impl<O: Object, DV: VariantImpl> VariantImplTable<O, DV> {
    pub fn new() -> Self {
        let object = OBJECT_REGISTRY.object_descriptor::<O>();
        VariantImplTable {
            variants: (0..object.discriminant_names().len())
                .map(|i| object.variant_impl(i))
                .collect(),
            phantom: PhantomData,
        }
    }
}

impl<O: Object, DV: VariantImpl> Index<usize> for VariantImplTable<O, DV> {
    type Output = &'static DV;
    fn index(&self, index: usize) -> &Self::Output {
        &self.variants[index]
    }
}

pub struct VariantImplSet(TypeMap);

impl VariantImplSet {
    pub(crate) fn new() -> Self {
        VariantImplSet(TypeMap::new())
    }
    pub fn insert<DV: VariantImpl>(&mut self, dv: DV) {
        self.0.insert(dv);
    }
    pub fn get<DV: VariantImpl>(&self) -> Option<&DV> {
        self.0.get::<DV>()
    }
}
