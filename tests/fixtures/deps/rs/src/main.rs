mod chain;
mod utils;

use chain::b::b_fn;
use utils::format::fmt;

fn main() {
    let _ = (b_fn(), fmt());
}
