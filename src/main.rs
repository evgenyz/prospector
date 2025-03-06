use prospector::lib_main;
use prospector::Options;

fn main() {
    let opts: Options = argh::from_env();
    lib_main(opts);
}
