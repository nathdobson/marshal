use marshal_pointer::empty::EmptyStrong;
use marshal_pointer::raw_arc::RawArc;
use marshal_pointer::raw_count::RawCount;
use marshal_pointer::raw_rc::RawRc;
use marshal_pointer::strong::Strong;
use marshal_pointer::unique::UniqueStrong;
use std::mem;

fn test<C: RawCount>() {
    {
        let strong = Strong::<C, _>::new(1);
        mem::drop(strong);
    }

    {
        let strong = Strong::<C, _>::new(1);
        let weak = Strong::downgrade(&strong);
        mem::drop(weak);
        mem::drop(strong);
    }

    {
        let strong = Strong::<C, _>::new(1);
        let weak = Strong::downgrade(&strong);
        mem::drop(strong);
        mem::drop(weak);
    }
    {
        UniqueStrong::<C, _>::new(1);
    }

    {
        let mut unique = UniqueStrong::<C, _>::new(1);
        *unique = 2;
        let weak = UniqueStrong::downgrade(&unique);
        assert!(weak.upgrade().is_none());
        let strong = UniqueStrong::into_strong(unique);
        assert!(weak.upgrade().is_some());
        mem::drop(strong);
        assert!(weak.upgrade().is_none());
    }

    {
        EmptyStrong::<C, u8>::new();
    }

    {
        let empty = EmptyStrong::<C, _>::new();
        empty.into_strong(1);
    }
}

#[test]
fn test_arc() {
    test::<RawArc>();
}

#[test]
fn test_rc() {
    test::<RawRc>();
}
