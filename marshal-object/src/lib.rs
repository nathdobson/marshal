#![feature(trait_alias)]
#![feature(const_type_name)]
#![feature(const_type_id)]
#![feature(unsize)]
#![feature(const_trait_impl)]
#![feature(coerce_unsized)]
#![feature(arbitrary_self_types)]

/// Serialize and deserialize trait objects, with type safety and monomorphization.
///
///
use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::Unsize;

use crate::de::DeserializeVariant;
use catalog::{Builder, BuilderFrom, Registry};
use type_map::concurrent::TypeMap;

pub mod de;
pub mod ser;

#[doc(hidden)]
pub mod reexports {
    pub use anyhow;
    pub use catalog;
    pub use marshal;
    pub use safe_once;
    pub use type_map;
}

pub trait AsDiscriminant<Key> {
    fn as_discriminant(self: *const Self) -> usize;
}

pub trait Object: 'static + Sized {
    type Dyn: ?Sized + AsDiscriminant<Self>;
    type Pointer<T: ?Sized>;
    fn object_descriptor() -> &'static ObjectDescriptor;
}

pub struct VariantDescriptor {
    variant_type: TypeId,
    variant_name: &'static str,
    deserializers: TypeMap,
}

pub struct ObjectDescriptor {
    variants: Vec<VariantDescriptor>,
    object_name: &'static str,
    discriminant_names: Option<Vec<&'static str>>,
    index_by_name: Option<HashMap<&'static str, usize>>,
    index_by_type: Option<HashMap<TypeId, usize>>,
}

impl ObjectDescriptor {
    pub fn object_name(&self) -> &'static str {
        self.object_name
    }
    pub fn discriminant_names(&self) -> &[&'static str] {
        self.discriminant_names.as_ref().unwrap()
    }
    pub fn variant_index_of(&self, s: &str) -> Option<usize> {
        self.index_by_name.as_ref().unwrap().get(s).copied()
    }
    pub fn deserialize_variant<DV: 'static + DeserializeVariant>(&self, index: usize) -> &DV {
        self.variants[index]
            .deserializers
            .get::<DV>()
            .unwrap_or_else(|| {
                panic!(
                    "cannot find deserializer for object `{}' variant `{}' of type `{}'",
                    self.object_name,
                    self.variants[index].variant_name,
                    type_name::<DV>()
                )
            })
    }
}

pub struct ObjectRegistry {
    objects: HashMap<TypeId, ObjectDescriptor>,
}

impl ObjectRegistry {
    pub fn object_descriptor<O: Object>(&self) -> &ObjectDescriptor {
        self.objects
            .get(&TypeId::of::<O>())
            .unwrap_or_else(|| panic!("Cannot find object descriptor for {}", type_name::<O>()))
    }
}

impl Builder for ObjectRegistry {
    type Output = Self;

    fn new() -> Self {
        ObjectRegistry {
            objects: HashMap::new(),
        }
    }

    fn build(mut self) -> Self::Output {
        for (_, object) in &mut self.objects {
            let mut variant_names = object
                .variants
                .iter()
                .map(|x| x.variant_name)
                .collect::<Vec<_>>();
            variant_names.shrink_to_fit();
            object.discriminant_names = Some(variant_names);

            let index_by_name = object
                .variants
                .iter()
                .enumerate()
                .map(|(i, v)| (v.variant_name, i))
                .collect();
            object.index_by_name = Some(index_by_name);

            let index_by_type = object
                .variants
                .iter()
                .enumerate()
                .map(|(i, v)| (v.variant_type, i))
                .collect();
            object.index_by_type = Some(index_by_type);
        }
        self
    }
}

pub static OBJECT_REGISTRY: Registry<ObjectRegistry> = Registry::new();

pub struct VariantRegistration {
    object_type: TypeId,
    object_name: &'static str,
    variant_type: TypeId,
    discriminant_name: &'static str,
    deserializers: fn(&mut TypeMap),
}

impl VariantRegistration {
    pub const fn new<O: Object, V: 'static>(deserializers: fn(&mut TypeMap)) -> Self
    where
        V: Unsize<O::Dyn>,
    {
        let variant_name = type_name::<V>();
        VariantRegistration {
            object_type: TypeId::of::<O>(),
            object_name: type_name::<O>(),
            variant_type: TypeId::of::<V>(),
            discriminant_name: variant_name,
            deserializers,
        }
    }
    pub fn object_type(&self) -> TypeId {
        self.object_type
    }
    pub fn object_name(&self) -> &'static str {
        self.object_name
    }
    pub fn variant_type(&self) -> TypeId {
        self.variant_type
    }
    pub fn discriminant_name(&self) -> &'static str {
        self.discriminant_name
    }
}

impl BuilderFrom<&'static VariantRegistration> for ObjectRegistry {
    fn insert(&mut self, element: &'static VariantRegistration) {
        let object = self
            .objects
            .entry(element.object_type)
            .or_insert_with(|| ObjectDescriptor {
                variants: vec![],
                object_name: element.object_name,
                discriminant_names: None,
                index_by_name: None,
                index_by_type: None,
            });
        let mut deserializers = TypeMap::new();
        (element.deserializers)(&mut deserializers);
        object.variants.push(VariantDescriptor {
            variant_type: element.variant_type,
            variant_name: element.discriminant_name,
            deserializers,
        });
    }
}

#[macro_export]
macro_rules! derive_variant {
    ($carrier:path, $concrete:ty) => {
        const _: () = {
            #[$crate::reexports::catalog::register(OBJECT_REGISTRY)]
            pub static REGISTER: VariantRegistration = VariantRegistration::new::<
                $carrier,
                $concrete,
            >(|map| {
                <$carrier as $crate::de::DeserializeVariantProvider<$concrete>>::add_deserialize_variant(map);
            });
            pub static VARIANT_INDEX: LazyLock<usize> = LazyLock::new(|| {
                OBJECT_REGISTRY
                    .object_descriptor::<$carrier>()
                    .variant_index_of(REGISTER.discriminant_name())
                    .unwrap()
            });
            impl AsDiscriminant<$carrier> for $concrete {
                fn as_discriminant(self:*const Self) -> usize {
                    *VARIANT_INDEX
                }
            }
        };
    };
}

#[macro_export]
macro_rules! derive_object {
    ($carrier:ident, $ptr_arg:ident, $ptr:ty, $tr:ident $(, $format:ident)*) => {
        const _: () = {
            $( $format!($carrier); )*
            impl $crate::de::DeserializeProvider for $carrier {}
            impl<V: 'static> $crate::de::DeserializeVariantProvider<V> for $carrier
            where $(
                $format::FormatDeserializeProvider::<$carrier> : $crate::de::DeserializeVariantProvider<V>,
            )*{
                fn add_deserialize_variant(
                    map: &mut $crate::reexports::type_map::concurrent::TypeMap,
                ) {
                    $(
                        <$format::FormatDeserializeProvider::<$carrier> as $crate::de::DeserializeVariantProvider<V>>::add_deserialize_variant(map);
                    )*
                }
            }

            impl $crate::Object for $carrier {
                type Dyn = dyn $tr;
                type Pointer<$ptr_arg:?Sized> = $ptr;
                fn object_descriptor() -> &'static ObjectDescriptor {
                    static ENTRY: LazyLock<&'static ObjectDescriptor> = LazyLock::new(|| {
                        OBJECT_REGISTRY.object_descriptor::<$carrier>()
                    });
                    *ENTRY
                }
            }


        };
    };
}

#[macro_export]
macro_rules! derive_box_object {
    ($carrier:ident, $tr:ident $(, $format:ident)*) => {
        derive_object!($carrier, T, ::std::boxed::Box<T>, $tr $(, $format)* );
        impl<E: Encoder> Serialize<E> for Box<dyn $tr>
            where dyn $tr: Serialize<E>,
        {
            fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
                serialize_object::<$carrier,E>(&**self, e, ctx)
            }
        }
        impl<'de, D: Decoder<'de>> Deserialize<'de, D> for Box<dyn $tr>
        where
            $carrier: DeserializeVariantForDiscriminant<'de, D>,
        {
            fn deserialize(p: D::AnyDecoder<'_>, ctx: &mut Context) -> anyhow::Result<Self> {
                deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    }
}

#[macro_export]
macro_rules! derive_rc_object {
    ($carrier:ident, $tr:ident $(, $format:ident)*) => {
        derive_object!($carrier, T, ::std::rc::Rc<T>, $tr $(, $format)* );
        impl<E: Encoder> $crate::reexports::marshal::ser::rc::SerializeRc<E> for dyn $tr
            where dyn $tr: Serialize<E>,
        {
            fn serialize_rc(this: &Rc<Self>, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
                serialize_object::<$carrier,E>(&**this, e, ctx)
            }
        }
        impl<'de, D: Decoder<'de>> $crate::reexports::marshal::de::rc::DeserializeRc<'de, D> for dyn $tr
        where
            $carrier: DeserializeVariantForDiscriminant<'de, D>,
        {
            fn deserialize_rc<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<Rc<Self>> {
                deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    }
}

#[macro_export]
macro_rules! derive_arc_object {
    ($carrier:ident, $tr:ident $(, $format:ident)*) => {
        derive_object!($carrier, T, ::std::sync::Arc<T>, $tr $(, $format)* );
        impl<E: Encoder> $crate::reexports::marshal::ser::rc::SerializeArc<E> for dyn $tr
            where dyn $tr: Serialize<E>,
        {
            fn serialize_arc(this: &::std::sync::Arc<Self>, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
                serialize_object::<$carrier,E>(&**this, e, ctx)
            }
        }
        impl<'de, D: Decoder<'de>> $crate::reexports::marshal::de::rc::DeserializeArc<'de, D> for dyn $tr
        where
            $carrier: DeserializeVariantForDiscriminant<'de, D>,
        {
            fn deserialize_arc<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<::std::sync::Arc<Self>> {
                deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    }
}

#[macro_export]
macro_rules! derive_rc_weak_object {
    ($carrier:ident, $tr:ident $(, $format:ident)*) => {
        derive_object!($carrier, T, ::std::rc::Weak<T>, $tr $(, $format)* );
        impl<E: Encoder> $crate::reexports::marshal::ser::rc::SerializeRcWeak<E> for dyn $tr
            where dyn $tr: Serialize<E>,
        {
            fn serialize_rc_weak(this: &::std::rc::Weak<Self>, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
                $crate::ser::serialize_rc_weak_object::<$carrier,E>(this, e, ctx)
            }
        }
        impl<'de, D: Decoder<'de>> $crate::reexports::marshal::de::rc::DeserializeRcWeak<'de, D> for dyn $tr
        where
            $carrier: DeserializeVariantForDiscriminant<'de, D>,
        {
            fn deserialize_rc_weak<'p>(p: D::AnyDecoder<'p>, ctx: &mut Context) -> anyhow::Result<::std::rc::Weak<Self>> {
                $crate::de::deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    }
}
