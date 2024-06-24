use std::fmt::Debug;

use marshal_derive::{Deserialize, Serialize};
use marshal_object::{
    AsDiscriminant, derive_arc_object, derive_box_object, derive_rc_object, derive_rc_weak_object,
};
use marshal_pointer::RawAny;

struct ArcFoo;
derive_arc_object!(ArcFoo, Foo, bin_object, json_object);
pub trait Foo: 'static + Debug + RawAny + AsDiscriminant<ArcFoo> {}

impl Foo for u8 {}
impl Foo for u16 {}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct A(u8);

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct B(u16);
