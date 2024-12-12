pub struct RO {}
pub struct WO {}
pub struct RW {}

pub struct Reg<T, A> {
    ptr: *mut u8,
    _phantom: ::core::marker::PhantomData<(*mut T, A)>
}

impl<T, A> Reg<T, A> {
    pub const unsafe fn from_ptr(ptr: *mut u8) -> Self {
        Self {
            ptr, _phantom: ::core::marker::PhantomData
        }
    }

    pub const fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }
}

impl<T: Copy> Reg<T, R> {
    pub unsafe fn read(&self) -> T {
        ::core::ptr::read_volatile(self.ptr as *mut T)
    }
}

impl<T: Copy> Reg<T, W> {
    pub unsafe fn write(&self, val: T) {
        ::core::ptr::write_volatile(self.ptr as *mut T, val)
    }
}

impl<T: Copy> Reg<T, RW> {
    pub unsafe fn read(&self) -> T {
        ::core::ptr::read_volatile(self.ptr as *mut T)
    }

    pub unsafe fn write(&self, val: T) {
        ::core::ptr::write_volatile(self.ptr as *mut T, val)
    }

    pub unsafe fn update(&self, f: impl FnOnce(T) -> T) {
        self.write(f(self.read()))
    }
}