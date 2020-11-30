// run-rustfix
// edition:2018

#![feature(async_closure)]
#![warn(clippy::async_yields_async)]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::thread::sleep;
use std::time::Duration;


#[rustfmt::skip]
fn main() {
    let _g = async {
        sleep(std::time::Duration::from_secs(1));
    };
}
