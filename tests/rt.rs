use core::task;
use core::pin::pin;
use core::future::Future;

pub(crate) mod thread {
    use std::thread::Thread;
    use core::{task, mem};

    const VTABLE: task::RawWakerVTable = task::RawWakerVTable::new(clone, wake, wake_by_ref, on_drop);

    unsafe fn on_drop(thread: *const ()) {
        let thread = Box::from_raw(thread as *mut Thread);
        drop(thread);
    }

    unsafe fn clone(thread: *const()) -> task::RawWaker {
        let thread = Box::from_raw(thread as *mut Thread);
        let new_ptr = thread.clone();
        mem::forget(thread);
        task::RawWaker::new(Box::into_raw(new_ptr) as _, &VTABLE)
    }

    unsafe fn wake(thread: *const ()) {
        let thread = Box::from_raw(thread as *mut () as *mut Thread);
        thread.unpark();
    }

    unsafe fn wake_by_ref(thread: *const ()) {
        let thread = &*(thread as *const Thread);
        thread.unpark();
    }

    #[inline(always)]
    pub fn waker(thread: Thread) -> task::Waker {
        //double pointer is so dumb...
        let thread = Box::new(thread);
        unsafe {
            task::Waker::from_raw(task::RawWaker::new(Box::into_raw(thread) as _, &VTABLE))
        }
    }
}

pub fn run<R, T: Future<Output = R>>(fut: T) -> R {
    let waker = thread::waker(std::thread::current());
    let mut ctx = task::Context::from_waker(&waker);

    let mut fut = pin!(fut);

    loop {
        match Future::poll(fut.as_mut(), &mut ctx) {
            task::Poll::Ready(result) => break result,
            task::Poll::Pending => std::thread::park(),
        }
    }
}
