#![no_std]
#![feature(async_await)]

use core::{
    cell::Cell,
    future::{Future, CONTEXT},
    pin::Pin,
    task::{Poll, Context},
};

struct SomeFuture(bool);

impl Future for SomeFuture {
    type Output = i32;
    #[inline(always)]
    fn poll(mut self: Pin<&mut Self>, _c: &mut Context) -> Poll<Self::Output> {
        if self.0 == false {
            self.0 = true;
            Poll::Pending
        } else {
            Poll::Ready(42)
        }
    }
}

#[inline(always)]
pub async fn test() -> i32 {
    let a: Cell<i32> = Cell::new(0);
    let b: &Cell<i32> = &a;
    SomeFuture(false).await;
    b.set(100);
    a.get()
}

#[allow(mutable_transmutes)]
#[inline(always)]
pub unsafe fn foo() -> i32 {
    let mut y = test();
    let mut x: Pin<&mut _> = Pin::new_unchecked(&mut y);

    // first time it returns pending:
    if let Poll::Pending = x.as_mut().poll(core::mem::transmute(&CONTEXT)) {
        // ok
    } else {
        core::hint::unreachable_unchecked()
    }

    // second time it returns ready::
    if let Poll::Ready(ref x) = x.as_mut().poll(core::mem::transmute(&CONTEXT)) {
        return *x;
    } else {
        core::hint::unreachable_unchecked()
    }
}
