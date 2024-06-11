#![feature(trait_alias)]
#![feature(const_type_name)]
#![feature(const_type_id)]
#![feature(unsize)]
#![feature(const_trait_impl)]

use std::any::{type_name, TypeId};
use std::collections::HashMap;

use catalog::{Builder, BuilderFrom, Registry};
use type_map::concurrent::TypeMap;

pub mod bin_format;
pub mod de;
pub mod json_format;
pub mod ser;

#[doc(hidden)]
pub mod reexports {
    pub use anyhow;
    pub use catalog;
    pub use safe_once;
    pub use type_map;
    pub use marshal_json;
    pub use marshal_bin;
    pub use marshal;
}

pub trait AsDiscriminant<Key> {
    fn as_discriminant(&self) -> usize;
}

pub trait Object: AsDiscriminant<Self::Key> {
    type Key;
    type Format;
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
    pub fn variant_deserializer<T: 'static>(&self, index: usize) -> &T {
        self.variants[index]
            .deserializers
            .get::<T>()
            .unwrap_or_else(|| {
                panic!(
                    "cannot find deserializer for object `{}' variant `{}' of type `{}'",
                    self.object_name,
                    self.variants[index].variant_name,
                    type_name::<T>()
                )
            })
    }
}

pub struct ObjectRegistry {
    objects: HashMap<TypeId, ObjectDescriptor>,
}

impl ObjectRegistry {
    pub fn object_descriptor(&self, id: TypeId) -> Option<&ObjectDescriptor> {
        self.objects.get(&id)
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
    deserializers: &'static [fn(&mut TypeMap)],
}

impl VariantRegistration {
    pub const fn new<T: 'static + ?Sized, V: 'static>(
        deserializers: &'static [fn(&mut TypeMap)],
    ) -> Self {
        let variant_name = type_name::<V>();
        VariantRegistration {
            object_type: TypeId::of::<T>(),
            object_name: type_name::<T>(),
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
        for f in element.deserializers {
            f(&mut deserializers);
        }
        object.variants.push(VariantDescriptor {
            variant_type: element.variant_type,
            variant_name: element.discriminant_name,
            deserializers,
        });
    }
}

#[macro_export]
macro_rules! define_variant {
    ($concrete:ty, $object:path) => {
        const _: () = {
            #[$crate::reexports::catalog::register(OBJECT_REGISTRY)]
            pub static REGISTER: VariantRegistration =
                VariantRegistration::new::<dyn $object, $concrete>(&[
                    // $(
                    // |map| <$format as $crate::de::VariantFormat<$concrete>>::add_object_deserializer::<dyn $object>(map)
                    // ),*
                    |map| {
                        <<dyn $object as $crate::Object>::Format as $crate::de::VariantFormat<
                            $concrete,
                        >>::add_object_deserializer::<dyn $object>(map)
                    },
                ]);
            pub static VARIANT_INDEX: LazyLock<usize> = LazyLock::new(|| {
                OBJECT_REGISTRY
                    .object_descriptor(REGISTER.object_type())
                    .unwrap()
                    .variant_index_of(REGISTER.discriminant_name())
                    .unwrap()
            });
            impl AsDiscriminant<<dyn $object as $crate::Object>::Key> for $concrete {
                fn as_discriminant(&self) -> usize {
                    *VARIANT_INDEX
                }
            }
        };
    };
}

#[macro_export]
macro_rules! derive_object {
    ($tr:ident, $parent:ident $(, $format:ident)*) => {
        pub struct Key;
        pub trait $parent = AsDiscriminant<Key> $( + $format::SerializeDyn )*;
        const _: () = {
            $( $format!($tr); )*
            pub struct CustomFormat;
            impl $crate::de::Format for CustomFormat {}
            impl<V: 'static> $crate::de::VariantFormat<V> for CustomFormat
            where $(
                $format::FormatType : $crate::de::VariantFormat<V>,
            )*{
                fn add_object_deserializer<T: 'static + ?Sized>(
                    map: &mut $crate::reexports::type_map::concurrent::TypeMap,
                ) where
                    V: ::std::marker::Unsize<T>,
                {
                    $(
                        <$format::FormatType as $crate::de::VariantFormat<V>>::add_object_deserializer::<T>(map);
                    )*
                }
            }

            impl $crate::Object for dyn $tr {
                type Key = Key;
                type Format = CustomFormat;
                fn object_descriptor() -> &'static ObjectDescriptor {
                    static ENTRY: LazyLock<&'static ObjectDescriptor> = LazyLock::new(|| {
                        OBJECT_REGISTRY
                            .object_descriptor(TypeId::of::<dyn $tr>())
                            .unwrap()
                    });
                    *ENTRY
                }
            }
            impl<E: Encoder> Serialize<E> for Box<dyn $tr>
            where
                dyn $tr: Serialize<E>,
            {
                fn serialize(&self, e: E::AnyEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
                    serialize_object(&**self, e, ctx)
                }
            }
            impl<'de, D: Decoder<'de>> Deserialize<'de, D> for Box<dyn $tr>
            where
                dyn $tr: DeserializeVariant<'de, D>,
            {
                fn deserialize(p: D::AnyDecoder<'_>, ctx: &mut Context) -> anyhow::Result<Self> {
                    deserialize_object::<D, dyn $tr>(p, ctx)
                }
            }
        };
    };
}
