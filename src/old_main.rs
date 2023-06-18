/*mod lib;
use lib::run;

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
fn main() {
    // block_on is passed a "future" that it waits on
    // run is an async function, so it returns a future immediately
    pollster::block_on(run());
}*/