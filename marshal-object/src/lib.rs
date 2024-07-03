#![feature(trait_alias)]
#![feature(const_type_name)]
#![feature(const_type_id)]
#![feature(unsize)]
#![feature(const_trait_impl)]
#![feature(coerce_unsized)]
#![feature(arbitrary_self_types)]
#![feature(trait_upcasting)]

/// Serialize and deserialize trait objects, with type safety and monomorphization.
///
///
use std::any::{type_name, TypeId};
use std::collections::HashMap;
use std::marker::Unsize;

use catalog::{Builder, BuilderFrom, Registry};

use marshal_pointer::AsFlatRef;

use crate::variants::{VariantImpl, VariantImplSet};

pub mod de;
pub mod ser;
pub mod variants;
pub mod type_id;

#[doc(hidden)]
pub mod reexports {
    pub use anyhow;
    pub use catalog;
    pub use safe_once;

    pub use marshal;
    pub use marshal_pointer;
    pub use paste;
}

pub trait AsDiscriminant<Key> {
    fn as_discriminant(self: *const Self) -> usize;
}

pub trait Object: 'static + Sized {
    type Dyn: ?Sized + AsDiscriminant<Self>;
    type Pointer<T: ?Sized>: AsFlatRef;
    fn object_descriptor() -> &'static ObjectDescriptor;
    fn discriminant_of(p: &<Self::Pointer<Self::Dyn> as AsFlatRef>::FlatRef) -> usize;
}

pub struct VariantDescriptor {
    variant_type: TypeId,
    variant_name: &'static str,
    deserializers: VariantImplSet,
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
    pub fn variant_impl<DV: 'static + VariantImpl>(&self, index: usize) -> &DV {
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
    deserializers: fn(&mut VariantImplSet),
}

impl VariantRegistration {
    pub const fn new<O: Object, V: 'static>(deserializers: fn(&mut VariantImplSet)) -> Self
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
        let mut deserializers = VariantImplSet::new();
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
        $crate::reexports::paste::paste! {
            const _: () = {
                #[$crate::reexports::catalog::register($crate::OBJECT_REGISTRY)]
                pub static [< VARIANT_ $concrete >]: $crate::VariantRegistration = $crate::VariantRegistration::new::<
                    $carrier,
                    $concrete,
                >(|map| {
                    <$carrier as $crate::ser::SerializeProvider<$concrete>>::add_serialize_variant(map);
                    <$carrier as $crate::de::DeserializeProvider<$concrete>>::add_deserialize_variant(
                        map,
                    );
                });
                pub static VARIANT_INDEX: $crate::reexports::safe_once::sync::LazyLock<usize> =
                    $crate::reexports::safe_once::sync::LazyLock::new(|| {
                        $crate::OBJECT_REGISTRY
                            .object_descriptor::<$carrier>()
                            .variant_index_of([< VARIANT_ $concrete >].discriminant_name())
                            .unwrap()
                    });
                impl $crate::AsDiscriminant<$carrier> for $concrete {
                    fn as_discriminant(self: *const Self) -> usize {
                        *VARIANT_INDEX
                    }
                }
            };
        }
    };
}

#[macro_export]
macro_rules! derive_serialize_provider {
    ($carrier:ident $(, $encoder: ty)* ) => {
        const _: () = {
            impl<V: 'static> $crate::ser::SerializeProvider<V> for $carrier where
                $(
                    <<$carrier as $crate::Object>::Pointer<V> as $crate::reexports::marshal_pointer::AsFlatRef>::FlatRef: $crate::reexports::marshal::ser::Serialize<$encoder>,
                )*
            {
                fn add_serialize_variant(
                    map: &mut $crate::variants::VariantImplSet,
                ) {
                    $(
                        map.insert(&::std::marker::PhantomData::<fn()->V> as &'static dyn $crate::ser::SerializeVariantDyn<$encoder, $carrier>);
                    )*
                }
            }
            $(
                impl $crate::ser::SerializeVariantForDiscriminant<$encoder> for $carrier {
                    fn serialize_variant<'w, 'en>(
                        this: &<Self::Pointer<Self::Dyn> as $crate::reexports::marshal_pointer::AsFlatRef>::FlatRef,
                        disc: usize,
                        e: $crate::reexports::marshal::encode::AnyEncoder<'w, 'en, $encoder>,
                        ctx: $crate::reexports::marshal::context::Context,
                    ) -> $crate::reexports::anyhow::Result<()> {
                        static SERIALIZERS: $crate::reexports::safe_once::sync::LazyLock<
                            $crate::variants::VariantImplTable<
                                $carrier,
                                &'static dyn $crate::ser::SerializeVariantDyn<$encoder, $carrier>
                            >
                        > =
                            $crate::reexports::safe_once::sync::LazyLock::new(
                                $crate::variants::VariantImplTable::new
                            );
                        SERIALIZERS[disc].serialize_variant_dyn(this, e, ctx)
                    }
                }
            )*
        };
    };
}

#[macro_export]
macro_rules! derive_deserialize_provider {
    ($carrier:ident $(, $decoder: ty)* ) => {
        const _: () = {
            impl<V: 'static> $crate::de::DeserializeProvider<V> for $carrier
            where
                $(
                    <$carrier as $crate::Object>::Pointer<V>: $crate::reexports::marshal::de::Deserialize<$decoder>,
                )*
                V: ::std::marker::Unsize<<$carrier as $crate::Object>::Dyn>,
            {
                fn add_deserialize_variant(
                    map: &mut $crate::variants::VariantImplSet,
                ) {
                    $(
                        map.insert(&::std::marker::PhantomData::<fn() -> V> as &'static dyn $crate::de::DeserializeVariantDyn<$decoder, $carrier>);
                    )*
                }
            }

            $(
                impl $crate::de::DeserializeVariantForDiscriminant<$decoder> for $carrier {
                    fn deserialize_variant<'p, 'de>(
                        disc: usize,
                        d: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, $decoder>,
                        ctx: $crate::reexports::marshal::context::Context
                    ) -> $crate::reexports::anyhow::Result<Self::Pointer<Self::Dyn>> {
                        static DESERIALIZERS: $crate::reexports::safe_once::sync::LazyLock<
                            $crate::variants::VariantImplTable<
                                $carrier,
                                &'static dyn $crate::de::DeserializeVariantDyn<$decoder, $carrier>
                            >
                        > =
                            $crate::reexports::safe_once::sync::LazyLock::new(
                                $crate::variants::VariantImplTable::new
                            );
                        DESERIALIZERS[disc].deserialize_variant_dyn(d, ctx)
                    }
                }
            )*
        };
    };
}

#[macro_export]
macro_rules! derive_object {
    ($carrier:ident, $ptr_arg:ident, $ptr:ty, $tr:ident) => {
        // $( $format!($carrier); )*
        const _: () = {
            impl $crate::Object for $carrier {
                type Dyn = dyn $tr;
                type Pointer<$ptr_arg: ?::std::marker::Sized> = $ptr;
                fn object_descriptor() -> &'static $crate::ObjectDescriptor {
                    static ENTRY: $crate::reexports::safe_once::sync::LazyLock<&'static $crate::ObjectDescriptor> = $crate::reexports::safe_once::sync::LazyLock::new(|| {
                        $crate::OBJECT_REGISTRY.object_descriptor::<$carrier>()
                    });
                    *ENTRY
                }
                fn discriminant_of(p: &<Self::Pointer<Self::Dyn> as $crate::reexports::marshal_pointer::AsFlatRef>::FlatRef) -> usize {
                    <Self::Dyn as $crate::AsDiscriminant<$carrier>>::as_discriminant(<<Self::Pointer<Self::Dyn> as $crate::reexports::marshal_pointer::AsFlatRef>::FlatRef as $crate::reexports::marshal_pointer::DerefRaw>::deref_raw(p))
                }
            }
        };
    };
}

#[macro_export]
macro_rules! derive_box_object {
    ($carrier:ident, $tr:ident) => {
        $crate::derive_object!($carrier, T, ::std::boxed::Box<T>, $tr);
        impl<E: $crate::reexports::marshal::encode::Encoder> $crate::reexports::marshal::ser::Serialize<E> for ::std::boxed::Box<dyn $tr>
            where $carrier: $crate::ser::SerializeVariantForDiscriminant<E>,
        {
            fn serialize<'w,'en>(&self, e: $crate::reexports::marshal::encode::AnyEncoder<'w,'en, E>, ctx: $crate::reexports::marshal::context::Context) -> $crate::reexports::anyhow::Result<()> {
                $crate::ser::serialize_object::<$carrier,E>(<::std::boxed::Box<dyn $tr> as $crate::reexports::marshal_pointer::AsFlatRef>::as_flat_ref(self), e, ctx)
            }
        }
        impl<D: $crate::reexports::marshal::decode::Decoder> $crate::reexports::marshal::de::Deserialize<D> for ::std::boxed::Box<dyn $tr>
        where
            $carrier: $crate::de::DeserializeVariantForDiscriminant<D>,
        {
            fn deserialize<'p, 'de>(p: $crate::reexports::marshal::decode::AnyDecoder<'p,'de,D>, ctx: $crate::reexports::marshal::context::Context) -> $crate::reexports::anyhow::Result<Self> {
                $crate::de::deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    }
}

#[macro_export]
macro_rules! derive_rc_object {
    ($carrier:ident, $tr:ident) => {
        $crate::derive_object!($carrier, T, ::std::rc::Rc<T>, $tr);
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeRc<E> for dyn $tr
        where
            $carrier: $crate::ser::SerializeVariantForDiscriminant<E>,
        {
            fn serialize_rc<'w, 'en>(
                this: &$crate::reexports::marshal_pointer::rc_ref::RcRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'w, 'en, E>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                $crate::ser::serialize_object::<$carrier, E>(this, e, ctx)
            }
        }
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeRc<D> for dyn $tr
        where
            $carrier: $crate::de::DeserializeVariantForDiscriminant<D>,
        {
            fn deserialize_rc<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::rc::Rc<Self>> {
                $crate::de::deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_arc_object {
    ($carrier:ident, $tr:ident) => {
        $crate::derive_object!($carrier, T, ::std::sync::Arc<T>, $tr);
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeArc<E> for dyn $tr
        where
            $carrier: $crate::ser::SerializeVariantForDiscriminant<E>,
        {
            fn serialize_arc<'w, 'en>(
                this: &$crate::reexports::marshal_pointer::arc_ref::ArcRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'w, 'en, E>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                //serialize_object::<$carrier,E>(&**this, e, ctx)
                ::std::todo!("X");
            }
        }
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeArc<D> for dyn $tr
        where
            $carrier: $crate::de::DeserializeVariantForDiscriminant<D>,
        {
            fn deserialize_arc<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Arc<Self>> {
                $crate::de::deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_rc_weak_object {
    ($carrier:ident, $tr:ident) => {
        $crate::derive_object!($carrier, T, ::std::rc::Weak<T>, $tr);
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeRcWeak<E> for dyn $tr
        where
            dyn $tr: $crate::reexports::marshal::ser::Serialize<E>,
        {
            fn serialize_rc_weak<'w, 'en>(
                this: &$crate::reexports::marshal_pointer::rc_weak_ref::RcWeakRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'w, 'en, E>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                ::std::todo!();
                // $crate::ser::serialize_rc_weak_object::<$carrier,E>(this, e, ctx)
            }
        }
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeRcWeak<D> for dyn $tr
        where
            $carrier: $crate::de::DeserializeVariantForDiscriminant<D>,
        {
            fn deserialize_rc_weak<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::rc::Weak<Self>> {
                $crate::de::deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    };
}

#[macro_export]
macro_rules! derive_arc_weak_object {
    ($carrier:ident, $tr:ident) => {
        $crate::derive_object!($carrier, T, ::std::sync::Weak<T>, $tr);
        impl<E: $crate::reexports::marshal::encode::Encoder>
            $crate::reexports::marshal::ser::rc::SerializeArcWeak<E> for dyn $tr
        where
            dyn $tr: $crate::reexports::marshal::ser::Serialize<E>,
        {
            fn serialize_arc_weak<'w, 'en>(
                this: &$crate::reexports::marshal_pointer::arc_weak_ref::ArcWeakRef<Self>,
                e: $crate::reexports::marshal::encode::AnyEncoder<'w, 'en, E>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<()> {
                ::std::todo!();
                // $crate::ser::serialize_rc_weak_object::<$carrier,E>(this, e, ctx)
            }
        }
        impl<D: $crate::reexports::marshal::decode::Decoder>
            $crate::reexports::marshal::de::rc::DeserializeArcWeak<D> for dyn $tr
        where
            $carrier: $crate::de::DeserializeVariantForDiscriminant<D>,
        {
            fn deserialize_arc_weak<'p, 'de>(
                p: $crate::reexports::marshal::decode::AnyDecoder<'p, 'de, D>,
                ctx: $crate::reexports::marshal::context::Context,
            ) -> $crate::reexports::anyhow::Result<::std::sync::Weak<Self>> {
                $crate::de::deserialize_object::<$carrier, D>(p, ctx)
            }
        }
    };
}
