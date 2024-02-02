use std::alloc::Layout;
use std::{alloc, ptr};
use std::mem::{MaybeUninit, size_of};

trait Map<T1> {
    type Target<T2>;
    fn map<T2>(self, f: impl FnMut(T1) -> T2) -> Self::Target<T2>;
}

impl<T1> Map<T1> for Box<T1> {
    type Target<T2> = Box<T2>;

    fn map<T2>(self, f: impl FnMut(T1) -> T2) -> Self::Target<T2> {
        // Get layouts of the types
        let from_layout = Layout::new::<T1>();
        let to_layout = Layout::new::<T2>();

        // If `T1` or `T2` is a ZST, we let `Box` handle the logic for us
        // If only T1 is a ZST, this will alloc
        // If only T2 is a ZST, this will dealloc
        // If both are a ZST, this will be a noop
        // In all cases, this is optimal
        if from_layout.size() == 0 || to_layout.size() == 0 {
            return Box::new(f(*self))
        }

        // Safety: Read T1, safe since `from_ptr` was created from a Box<T1>
        let from_ptr = Box::into_raw(self);
        let v = unsafe { ptr::read(from_ptr) };

        // Apply `f`. Create a temporary Box so the allocation of the Box is deallocated on panic of `f`
        let tmp_box: Box<MaybeUninit<T1>> = unsafe { Box::from_raw(from_ptr) };
        let v = f(v);
        Box::into_raw(tmp_box);

        // Generate a `to_ptr` from `from_ptr` that can fit a `T2`
        let to_ptr = if from_ptr as usize % to_layout.align() != 0 {
            // We need to re-allocate, because the alignment of the Box is insufficient
            unsafe {
                // Safety: from_ptr was created from a Box with from_layout and
                alloc::dealloc(from_ptr, from_layout);
                alloc::alloc(to_layout) as *mut T2
            }
        } else if to_layout.size() != from_layout.size() {
            // We need to re-allocate, because the size of the Box is incorrect
            unsafe {
                alloc::realloc(from_ptr, from_layout, to_layout.size()) as *mut T2
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

