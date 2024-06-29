use marshal_core::encode::{AnyGenEncoder,  GenEncoder};

use crate::context::Context;
use crate::ser::Serialize;

impl<'a, E: GenEncoder, T: Serialize<E>> Serialize<E> for &'a T {
    fn serialize<'w, 'en>(&self, e: AnyGenEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        (**self).serialize(e, ctx)
    }
}
