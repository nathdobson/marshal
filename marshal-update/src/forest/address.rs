#[derive(Eq, Ord, PartialEq, PartialOrd, Hash)]
pub(crate) struct Address(*const ());

unsafe impl Sync for Address {}
unsafe impl Send for Address {}
