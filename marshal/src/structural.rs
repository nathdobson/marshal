pub struct StructNil<const S: &'static str> {}

type Foo = StructNil<"Foo">;

pub trait Structural {}

impl<const S: &'static str> Structural for StructNil<S> {}
