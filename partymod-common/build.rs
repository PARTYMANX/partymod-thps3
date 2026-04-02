fn main() {
    cc::Build::new()
        .file("src/console/stdstream.c")
        .compile("stdstream");
}
