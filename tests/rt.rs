use core::task;
use core::pin::pin;
use core::future::Future;

pub(crate) mod thread {
    use std::thread::Thread;
    use core::{task, mem};

    const VTABLE: task::RawWakerVTable = task::RawWakerVTable::new(clone, action, action, on_drop);

    unsafe fn on_drop(thread: *const ()) {
        let thread: Thread = mem::transmute(thread);
        drop(thread);
    }

    unsafe fn clone(thread: *const()) -> task::RawWaker {
        let thread: Thread = mem::transmute(thread);
        let new_ptr = mem::transmute(thread.clone());
        mem::forget(thread);
        task::RawWaker::new(new_ptr, &VTABLE)
    }

    unsafe fn action(thread: *const ()) {
        let thread: Thread = mem::transmute(thread);
        thread.unpark();
    }

    #[inline(always)]
    pub fn waker(thread: Thread) -> task::Waker {
        unsafe {
            task::Waker::from_raw(task::RawWaker::new(mem::transmute(thread), &VTABLE))
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
