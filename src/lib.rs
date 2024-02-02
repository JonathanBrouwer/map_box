use std::alloc::Layout;
use std::mem::MaybeUninit;
use std::{alloc, ptr};

pub trait Map<T1> {
    type Target<T2>;

    /// Returns a box, with function f applied to the value inside.
    /// This function will re-use the allocation when possible.
    fn map<T2>(self, f: impl FnMut(T1) -> T2) -> Self::Target<T2>;
}

impl<T1> Map<T1> for Box<T1> {
    type Target<T2> = Box<T2>;

    /// Returns a box, with function f applied to the value inside.
    /// This function will re-use the allocation when possible.
    fn map<T2>(self, mut f: impl FnMut(T1) -> T2) -> Self::Target<T2> {
        // Get layouts of the types
        let from_layout = Layout::new::<T1>();
        let to_layout = Layout::new::<T2>();

        // If `T1` or `T2` is a ZST, we let `Box` handle the logic for us. This is optimal.
        // If the alignment requirements of T1 and T2 are different, we also call this.
        // This is because `dealloc` requires the alignment to be identical, and there's no way to realloc with a different Layout
        if from_layout.size() == 0
            || to_layout.size() == 0
            || from_layout.align() != to_layout.align()
        {
            return Box::new(f(*self));
        }

        // Safety: Read T1, safe since `from_ptr` was created from a Box<T1>
        let from_ptr = Box::into_raw(self);
        let v = unsafe { ptr::read(from_ptr) };

        // Apply `f`. Create a temporary Box so the allocation of the Box is deallocated on panic of `f`
        let tmp_box: Box<MaybeUninit<T1>> =
            unsafe { Box::from_raw(from_ptr as *mut MaybeUninit<T1>) };
        let v = f(v);
        Box::into_raw(tmp_box);

        // Generate a `to_ptr` from `from_ptr` that can fit a `T2`
        let to_ptr = if to_layout.size() != from_layout.size() {
            // We need to re-allocate, because the size of the Box is incorrect
            unsafe {
                // Safety: from_layout was used to allocate the Box and to_layout is non-zero
                alloc::realloc(from_ptr as *mut u8, from_layout, to_layout.size()) as *mut T2
            }
        } else {
            // Size and alignment are correct, so we can re-use the allocation
            from_ptr as *mut T2
        };

        unsafe {
            // Safety: The logic above guarantees that to_ptr can fit a `T2`
            ptr::write(to_ptr, v);
            Box::from_raw(to_ptr)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Map;

    #[test]
    fn same_type() {
        let b = Box::new(21u64);
        let b = b.map(|v| v * 2);
        assert_eq!(*b, 42);
    }

    #[test]
    fn same_size() {
        let b = Box::new(42u64);
        let b = b.map(|v| v as i64);
        assert_eq!(*b, 42);
    }

    #[test]
    fn down_size() {
        let b = Box::new(42u64);
        let b = b.map(|v| v as u32);
        assert_eq!(*b, 42);
    }

    #[test]
    fn up_size() {
        let b = Box::new(42u32);
        let b = b.map(|v| v as u64);
        assert_eq!(*b, 42);
    }

    #[test]
    fn zst_in() {
        let b = Box::new(());
        let b = b.map(|_| 42u64);
        assert_eq!(*b, 42);
    }

    #[test]
    fn zst_out() {
        let b = Box::new(42u64);
        let b = b.map(|_| ());
        assert_eq!(*b, ());
    }

    #[test]
    fn zst_both() {
        let b = Box::new(());
        let b: Box<[u64; 0]> = b.map(|_| []);
        assert_eq!(*b, []);
    }
}
