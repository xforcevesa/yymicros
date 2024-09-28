use core::cell::{RefCell, RefMut};
use core::cell::UnsafeCell;
use core::fmt;
use core::mem::MaybeUninit;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// A wrapper of a lazy initialized value.
///
/// It implements [`Deref`] and [`DerefMut`]. The caller must use the dereference
/// operation after initialization, otherwise it will panic.
pub struct LazyInit<T> {
    inited: AtomicBool,
    data: UnsafeCell<MaybeUninit<T>>,
}

unsafe impl<T: Send + Sync> Sync for LazyInit<T> {}
unsafe impl<T: Send> Send for LazyInit<T> {}

impl<T> LazyInit<T> {
    /// Creates a new uninitialized value.
    pub const fn new() -> Self {
        Self {
            inited: AtomicBool::new(false),
            data: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    /// Initializes the value.
    ///
    /// # Panics
    ///
    /// Panics if the value is already initialized.
    pub fn init_by(&self, data: T) {
        assert!(!self.is_init());
        unsafe { (*self.data.get()).as_mut_ptr().write(data) };
        self.inited.store(true, Ordering::Release);
    }

    /// Checks whether the value is initialized.
    pub fn is_init(&self) -> bool {
        self.inited.load(Ordering::Acquire)
    }

    /// Gets a reference to the value.
    ///
    /// Returns [`None`] if the value is not initialized.
    pub fn try_get(&self) -> Option<&T> {
        if self.is_init() {
            unsafe { Some(&*(*self.data.get()).as_ptr()) }
        } else {
            None
        }
    }

    fn check_init(&self) {
        if !self.is_init() {
            panic!(
                "Use uninitialized value: {:?}",
                core::any::type_name::<Self>()
            )
        }
    }

    #[inline]
    fn get(&self) -> &T {
        self.check_init();
        unsafe { self.get_unchecked() }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut T {
        self.check_init();
        unsafe { self.get_mut_unchecked() }
    }

    /// Gets the reference to the value without checking if it is initialized.
    ///
    /// # Safety
    ///
    /// Must be called after initialization.
    #[inline]
    pub unsafe fn get_unchecked(&self) -> &T {
        unsafe { &*(*self.data.get()).as_ptr() }
    }

    /// Get a mutable reference to the value without checking if it is initialized.
    ///
    /// # Safety
    ///
    /// Must be called after initialization.
    #[inline]
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        unsafe { &mut *(*self.data.get()).as_mut_ptr() }
    }
}

impl<T: fmt::Debug> fmt::Debug for LazyInit<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_get() {
            Some(s) => write!(f, "LazyInit {{ data: ")
                .and_then(|()| s.fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "LazyInit {{ <uninitialized> }}"),
        }
    }
}

impl<T> Deref for LazyInit<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        self.get()
    }
}

impl<T> DerefMut for LazyInit<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut T {
        self.get_mut()
    }
}

impl<T> Drop for LazyInit<T> {
    fn drop(&mut self) {
        if self.is_init() {
            unsafe { core::ptr::drop_in_place((*self.data.get()).as_mut_ptr()) };
        }
    }
}

/// Wrap a static data structure inside it so that we are
/// able to access it without any `unsafe`.
///
/// We should only use it in uniprocessor.
///
/// In order to get mutable reference of inner data, call
/// `exclusive_access`.
#[derive(Clone)]
pub struct UPSafeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// User is responsible to guarantee that inner struct is only used in
    /// uniprocessor.
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }
    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
