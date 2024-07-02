// impl<D: Decoder, T: ?Sized + DeserializeUpdateDyn<D>> DeserializeUpdate<D> for Box<T>
// where
//     Box<T>: Deserialize<D>,
// {
//     fn deserialize_update<'p, 'de>(
//         &mut self,
//         d: AnyDecoder<'p, 'de, D>,
//         ctx: Context,
//     ) -> anyhow::Result<()> {
//         (**self).deserialize_update_dyn(d, ctx)
//     }
// }
