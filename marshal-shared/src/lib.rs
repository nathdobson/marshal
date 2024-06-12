mod de;
mod ser;

use marshal::context::Context;
use marshal::encode::{AnyEncoder, Encoder, TupleVariantEncoder};
use marshal::reexports::anyhow;
use marshal::reexports::anyhow::anyhow;
use marshal::ser::Serialize;
use std::any::Any;
use std::rc;
use std::rc::{Rc, Weak};
use weak_table::ptr_weak_key_hash_map::Entry;
use weak_table::PtrWeakKeyHashMap;


