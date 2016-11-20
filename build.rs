

extern crate gcc;


const LIB: &'static str = "src/linux64.c";

fn main() {
    gcc::Config::new()
        .file(LIB)
        .include("src")
        .compile("libexternal.a");
}
