use std::rc;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder};
use marshal::ser::Serialize;
use marshal_pointer::AsFlatRef;

use crate::{AsDiscriminant, Object};

pub fn serialize_object<'w, 'en, O: Object, E: Encoder>(
    value: &<O::Pointer<O::Dyn> as AsFlatRef>::FlatRef,
    e: AnyEncoder<'w, 'en, E>,
    ctx: Context,
) -> anyhow::Result<()>
where
    O: SerializeVariantForDiscriminant<E>,
{
    let disc = O::discriminant_of(value);
    let mut e = e.encode_tuple_variant(
        O::object_descriptor().object_name(),
        O::object_descriptor().discriminant_names(),
        disc,
        1,
    )?;
    O::serialize_variant(value, disc, e.encode_field()?, ctx)?;
    e.end()?;
    Ok(())
}

pub fn serialize_rc_weak_object<'w, 'en, O: Object, E: Encoder>(
    value: &rc::Weak<O::Dyn>,
    e: AnyEncoder<'w, 'en, E>,
    ctx: Context,
) -> anyhow::Result<()>
where
    O::Dyn: Serialize<E>,
{
    let mut e = e.encode_tuple_variant(
        O::object_descriptor().object_name(),
        O::object_descriptor().discriminant_names(),
        value.as_ptr().as_discriminant(),
        1,
    )?;
    if let Some(value) = value.upgrade() {
        let mut e = e.encode_field()?.encode_some()?;
        <O::Dyn as Serialize<E>>::serialize(&value, e.encode_some()?, ctx)?;
        e.end()?;
    } else {
        e.encode_field()?.encode_none()?;
    }
    e.end()?;
    Ok(())
}

pub trait SerializeVariantForDiscriminant<E: Encoder>: Object {
    fn serialize_variant<'w, 'en>(
        this: &<Self::Pointer<Self::Dyn> as AsFlatRef>::FlatRef,
        disc: usize,
        e: AnyEncoder<'w, 'en, E>,
        ctx: Context,
    ) -> anyhow::Result<()>;
}

// pub trait DowncastSerialize<V, E: Encoder> {
//     fn downcast_serialize(&self, e: E::AnySpecEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()>;
// }
//
// impl<T: ?Sized + Unsize<dyn Any>, V: 'static + Serialize<E>, E: Encoder> DowncastSerialize<V, E>
//     for Box<T>
// {
//     fn downcast_serialize(&self, e: E::AnySpecEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
//         (&**self as &dyn Any)
//             .downcast_ref::<V>()
//             .unwrap()
//             .serialize(e, ctx)
//     }
// }
//
// impl<T: ?Sized + RawAny, V: 'static + Serialize<E>, E: Encoder> DowncastSerialize<V, E> for Arc<T> {
//     fn downcast_serialize(&self, e: E::AnySpecEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
//         if Arc::as_ptr(self).raw_type_id() == TypeId::of::<V>() {
//             unsafe { Arc::from_raw(Arc::into_raw(self.clone()) as *const V).serialize(e, ctx) }
//         } else {
//             panic!("cannot downcast to {}", type_name::<V>());
//         }
//     }
// }
//
// impl<T: ?Sized + Unsize<dyn Any>, V: 'static + Serialize<E>, E: Encoder> DowncastSerialize<V, E>
//     for Rc<T>
// {
//     fn downcast_serialize(&self, e: E::AnySpecEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
//         Rc::<dyn 'static + Any>::downcast::<V>(self.clone())
//             .unwrap()
//             .serialize(e, ctx)
//     }
// }
//
// impl<T: ?Sized + RawAny, V: 'static + Serialize<E>, E: Encoder> DowncastSerialize<V, E>
//     for rc::Weak<T>
// {
//     fn downcast_serialize(&self, e: E::AnySpecEncoder<'_>, ctx: &mut Context) -> anyhow::Result<()> {
//         if self.as_ptr().raw_type_id() == TypeId::of::<V>() {
//             unsafe { Rc::from_raw(self.clone().into_raw() as *const V).serialize(e, ctx) }
//         } else {
//             panic!("cannot downcast to {}", type_name::<V>());
//         }
//     }
// }
