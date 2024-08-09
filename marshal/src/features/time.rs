use crate::context::Context;
use crate::de::Deserialize;
use crate::ser::Serialize;
use marshal_core::decode::{AnyDecoder, Decoder};
use marshal_core::encode::{AnyEncoder, Encoder};
use std::time::{Duration, Instant, SystemTime};

impl<E: Encoder> Serialize<E> for Duration {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        <(u64, u32) as Serialize<E>>::serialize(&(self.as_secs(), self.subsec_nanos()), e, ctx)
    }
}

impl<D: Decoder> Deserialize<D> for Duration {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        let (secs, nanos) = <(u64, u32) as Deserialize<D>>::deserialize(d, ctx)?;
        Ok(Duration::new(secs, nanos))
    }
}

impl<E: Encoder> Serialize<E> for SystemTime {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        <Duration as Serialize<E>>::serialize(
            &self.duration_since(SystemTime::UNIX_EPOCH).unwrap(),
            e,
            ctx,
        )
    }
}

impl<D: Decoder> Deserialize<D> for SystemTime {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        Ok(SystemTime::UNIX_EPOCH
            .checked_add(<Duration as Deserialize<D>>::deserialize(d, ctx)?)
            .unwrap())
    }
}

impl<E: Encoder> Serialize<E> for Instant {
    fn serialize<'w, 'en>(&self, e: AnyEncoder<'w, 'en, E>, ctx: Context) -> anyhow::Result<()> {
        let now = Instant::now();
        let delta = if let Some(future) = self.checked_duration_since(now) {
            (future.as_secs() as i64, future.subsec_nanos())
        } else {
            let past = now.checked_duration_since(*self).unwrap();
            (-(past.as_secs() as i64), past.subsec_nanos())
        };
        <(i64, u32) as Serialize<E>>::serialize(&delta, e, ctx)?;
        Ok(())
    }
}

impl<D: Decoder> Deserialize<D> for Instant {
    fn deserialize<'p, 'de>(d: AnyDecoder<'p, 'de, D>, ctx: Context) -> anyhow::Result<Self> {
        let (secs, nanos) = <(i64, u32) as Deserialize<D>>::deserialize(d, ctx)?;
        let now = Instant::now();
        if secs < 0 {
            Ok(now - Duration::new((-secs) as u64, nanos))
        } else {
            Ok(now + Duration::new(secs as u64, nanos))
        }
    }
}
