// This mod MUST go first, so that the others see its macros.
pub(crate) mod fmt;

mod block_on;
mod yield_now;

pub mod join;
pub mod select;

#[allow(unused_imports)]
pub use block_on::*;
#[allow(unused_imports)]
pub use yield_now::*;

use crate::time::get_time_ms;

fn sleep(dur: u64) {
    let start = get_time_ms();
    loop {
        if ((get_time_ms() - start) as u64) >= dur {
            break;
        }
    }
}

pub fn futures_test() {
    let fut = join::join(
        async {
            sleep(1000);
            println!("Hello after 2 seconds");
        },
        async {
            sleep(1000);
            println!("Hello after 2 seconds");
        }
    );
    block_on(fut);
}
