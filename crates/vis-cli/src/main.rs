mod app;
mod error;

use std::env;

fn main() {
    app::exit(app::run(env::args().skip(1)));
}
