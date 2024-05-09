use core::ffi::c_void;
use core::{ptr, task, hint, mem};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use alloc::boxed::Box;

use crate::error::{error, ErrorCode};
use crate::msg::Message;

use nng_c_sys as sys;

mod noop {
    use core::{ptr, task};

    #[cold]
    fn should_not_clone(_: *const()) -> task::RawWaker {
        panic!("Impossible Waker Clone");
    }

    static VTABLE: task::RawWakerVTable = task::RawWakerVTable::new(should_not_clone, action, action, action);

    pub fn action(_: *const ()) {
    }

    #[inline(always)]
    pub fn waker() -> task::Waker {
        unsafe {
            task::Waker::from_raw(task::RawWaker::new(ptr::null(), &VTABLE))
        }
    }
}

/// Idle state
const WAITING: u8 = 0;

/// A new waker value is being registered with the `AtomicWaker` cell.
const REGISTERING: u8 = 0b01;

/// The waker currently registered with the `AtomicWaker` cell is being woken.
const WAKING: u8 = 0b10;

#[doc(hidden)]
/// Atomic waker used by `TimerState`
pub struct AtomicWaker {
    state: AtomicU8,
    waker: UnsafeCell<task::Waker>,
}

struct StateRestore<F: Fn()>(F);
impl<F: Fn()> Drop for StateRestore<F> {
    fn drop(&mut self) {
        (self.0)()
    }
}

macro_rules! impl_register {
    ($this:ident($waker:ident) { $($impl:tt)+ }) => {
        match $this.state.compare_exchange(WAITING, REGISTERING, Ordering::Acquire, Ordering::Acquire).unwrap_or_else(|err| err) {
            WAITING => {
                //Make sure we do not stuck in REGISTERING state
                let state_guard = StateRestore(|| {
                    $this.state.store(WAITING, Ordering::Release);
                });

                unsafe {
                    $(
                        $impl
                    )+

                    // Release the lock. If the state transitioned to include
                    // the `WAKING` bit, this means that a wake has been
                    // called concurrently, so we have to remove the waker and
                    // wake it.`
                    //
                    // Start by assuming that the state is `REGISTERING` as this
                    // is what we jut set it to.
                    match $this.state.compare_exchange(REGISTERING, WAITING, Ordering::AcqRel, Ordering::Acquire) {
                        Ok(_) => {
                            mem::forget(state_guard);
                        }
                        Err(actual) => {
                            // This branch can only be reached if a
                            // concurrent thread called `wake`. In this
                            // case, `actual` **must** be `REGISTERING |
                            // `WAKING`.
                            debug_assert_eq!(actual, REGISTERING | WAKING);

                            let mut waker = noop::waker();
                            ptr::swap($this.waker.get(), &mut waker);

                            // Just restore state,
                            // because no one could change state while state == `REGISTERING` | `WAKING`.
                            drop(state_guard);
                            waker.wake();
                        }
                    }
                }
            }
            WAKING => {
                // Currently in the process of waking the task, i.e.,
                // `wake` is currently being called on the old task handle.
                // So, we call wake on the new waker
                $waker.wake_by_ref();
                hint::spin_loop();
            }
            state => {
                // In this case, a concurrent thread is holding the
                // "registering" lock. This probably indicates a bug in the
                // caller's code as racing to call `register` doesn't make much
                // sense.
                //
                // We just want to maintain memory safety. It is ok to drop the
                // call to `register`.
                debug_assert!(
                    state == REGISTERING ||
                    state == REGISTERING | WAKING
                );
            }
        }
    };
}

impl AtomicWaker {
    fn new() -> Self {
        Self {
            state: AtomicU8::new(WAITING),
            waker: UnsafeCell::new(noop::waker()),
        }
    }

    #[allow(clippy::assigning_clones)]
    fn register_ref(&self, waker: &task::Waker) {
        impl_register!(self(waker) {
            // Lock acquired, update the waker cell
            if !(*self.waker.get()).will_wake(waker) {
                //Clone new waker if it is definitely not the same as old one
                *self.waker.get() = waker.clone();
            }
        });
    }

    fn wake(&self) {
        // AcqRel ordering is used in order to acquire the value of the `task`
        // cell as well as to establish a `release` ordering with whatever
        // memory the `AtomicWaker` is associated with.
        match self.state.fetch_or(WAKING, Ordering::AcqRel) {
            WAITING => {
                // The waking lock has been acquired.
                let mut waker = noop::waker();
                unsafe {
                    ptr::swap(self.waker.get(), &mut waker);
                }

                // Release the lock
                self.state.fetch_and(!WAKING, Ordering::Release);
                waker.wake();
            }
            state => {
                // There is a concurrent thread currently updating the
                // associated task.
                //
                // Nothing more to do as the `WAKING` bit has been set. It
                // doesn't matter if there are concurrent registering threads or
                // not.
                debug_assert!(
                    state == REGISTERING ||
                    state == REGISTERING | WAKING ||
                    state == WAKING
                );
            }
        }
    }
}

unsafe impl Send for AtomicWaker {}
unsafe impl Sync for AtomicWaker {}


struct State {
    ready: AtomicBool,
    waker: AtomicWaker,
    aio: *mut sys::nng_aio,
}

impl State {
    #[inline]
    pub(crate) fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    #[inline]
    ///Notifies underlying `Waker`
    ///
    ///After that `Waker` is no longer registered
    pub(crate) fn wake(&self) {
        if !self.ready.compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst).unwrap_or_else(|err| err) {
            self.waker.wake();
        }
    }
}

unsafe extern "C" fn aio_callback(data: *mut c_void) {
    //This should never happen unless we'll have need for no-data callback
    if data.is_null() {
        return;
    }

    //State will be always valid as operation will be cancelled before pointer is freed
    let state = &*(data as *mut State);
    state.wake();
}

#[repr(transparent)]
//Wrapper for nng's async struct
pub(crate) struct Aio {
    state: &'static mut State,
}

impl Aio {
    pub(crate) fn new() -> Result<Self, ErrorCode> {
        let state = Box::new(State {
            ready: AtomicBool::new(false),
            waker: AtomicWaker::new(),
            aio: ptr::null_mut(),

        });
        let state = Box::leak(state);
        let result = unsafe {
            sys::nng_aio_alloc(&mut state.aio, Some(aio_callback), state as *mut _ as *mut _)
        };

        match result {
            0 => Ok(Self {
                state,
            }),
            code => unsafe {
                let _ = Box::from_raw(state);
                Err(error(code))
            }
        }
    }

    #[inline(always)]
    pub(crate) fn is_ready(&self) -> bool {
        self.state.is_ready()
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut sys::nng_aio {
        self.state.aio
    }

    #[inline]
    pub(crate) fn register_waker(&self, waker: &task::Waker) {
        self.state.waker.register_ref(waker);
    }

    ///Returns operation status, assuming there is no message involved
    ///
    ///This obviously should not be used for futures that are receiving message
    pub(crate) fn get_send_result(&mut self) -> Result<(), (Message, ErrorCode)> {
        let result = unsafe {
            sys::nng_aio_result(self.state.aio)
        };

        if result != 0 {
            let msg = unsafe {
                sys::nng_aio_get_msg(self.state.aio)
            };
            unsafe {
                sys::nng_aio_set_msg(self.state.aio, ptr::null_mut());
            }
            let msg = ptr::NonNull::new(msg).expect("to have message");

            return Err((Message(msg), error(result)));
        }

        Ok(())
    }

    ///Extracts message from AIO, if any
    ///
    ///This obviously None, if operation involved no message receiving,
    ///but also if it is not complete yet
    pub(crate) fn get_msg(&mut self) -> Result<Option<Message>, ErrorCode> {
        if !self.state.is_ready() {
            return Err(error(sys::nng_errno_enum::NNG_EAGAIN));
        }

        let result = unsafe {
            sys::nng_aio_result(self.state.aio)
        };

        if result != 0 {
            return Err(error(result));
        }

        let result = unsafe {
            sys::nng_aio_get_msg(self.state.aio)
        };

        unsafe {
            sys::nng_aio_set_msg(self.state.aio, ptr::null_mut());
        }

        Ok(ptr::NonNull::new(result).map(Message))
    }
}

impl Drop for Aio {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            //This blocks until callback is called
            sys::nng_aio_stop(self.state.aio);

            //Make sure no message is leaked before we free AIO
            let msg = sys::nng_aio_get_msg(self.state.aio);
            if !msg.is_null() {
                sys::nng_msg_free(msg);
            }

            sys::nng_aio_free(self.state.aio);

            let _ = Box::from_raw(self.state);
        }
    }
}
