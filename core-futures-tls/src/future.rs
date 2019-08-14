//! Asynchronous values.

use core::marker::Unpin;
use core::ops::{Generator, GeneratorState};
use core::pin::Pin;
use core::task::{Context, Poll, Waker, RawWakerVTable, RawWaker};

unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}
unsafe fn clone(_: *const ()) -> RawWaker {
    RawWaker::new(core::ptr::null(), &VTABLE)
}


static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

#[allow(unions_with_drop_fields)]
union TransmuteRawWaker2Waker {
    w: Waker,
    rw: RawWaker
}
static WAKER: Waker = unsafe {
    TransmuteRawWaker2Waker { rw: RawWaker::new(core::ptr::null(), &VTABLE) }.w
};

union TransmuteWaker2Context {
    c: Context<'static>,
    w: &'static Waker
}

pub static CONTEXT: Context = unsafe { TransmuteWaker2Context { w: &WAKER }.c };

#[doc(inline)]
pub use core::future::*;

/// Wrap a generator in a future.
///
/// This function returns a `GenFuture` underneath, but hides it in `impl Trait` to give
/// better error messages (`impl Future` rather than `GenFuture<[closure.....]>`).
#[doc(hidden)]
pub fn from_generator<T: Generator<Yield = ()>>(x: T) -> impl Future<Output = T::Return> {
    GenFuture(x)
}

/// A wrapper around generators used to implement `Future` for `async`/`await` code.
#[doc(hidden)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct GenFuture<T: Generator<Yield = ()>>(T);

// We rely on the fact that async/await futures are immovable in order to create
// self-referential borrows in the underlying generator.
impl<T: Generator<Yield = ()>> !Unpin for GenFuture<T> {}

#[doc(hidden)]
impl<T: Generator<Yield = ()>> Future for GenFuture<T> {
    type Output = T::Return;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Safe because we're !Unpin + !Drop mapping to a ?Unpin value
        let gen = unsafe { Pin::map_unchecked_mut(self, |s| &mut s.0) };
        set_task_context(cx, || match gen.resume() {
            GeneratorState::Yielded(()) => Poll::Pending,
            GeneratorState::Complete(x) => Poll::Ready(x),
        })
    }
}

#[doc(hidden)]
/// Sets the thread-local task context used by async/await futures.
pub fn set_task_context<F, R>(_cx: &mut Context<'_>, f: F) -> R
where
    F: FnOnce() -> R,
{
    f()
}


#[doc(hidden)]
/// Retrieves the thread-local task context used by async/await futures.
///
/// This function acquires exclusive access to the task context.
///
/// Panics if no context has been set or if the context has already been
/// retrieved by a surrounding call to get_task_context.
pub fn get_task_context<F, R>(f: F) -> R
where
    F: FnOnce(&mut Context<'_>) -> R,
{
    #[allow(mutable_transmutes)]
    unsafe { f(core::mem::transmute(&CONTEXT)) }
}

#[doc(hidden)]
/// Polls a future in the current thread-local task waker.
pub fn poll_with_tls_context<F>(f: Pin<&mut F>) -> Poll<F::Output>
where
    F: Future,
{
    get_task_context(|cx| F::poll(f, cx))
}
