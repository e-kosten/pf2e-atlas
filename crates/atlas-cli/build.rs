fn main() {
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Reserve room for normal nested source records and CEL filters. MSVC
        // otherwise gives the main thread only 1 MiB. Pages commit on demand.
        println!("cargo::rustc-link-arg-bin=atlas=/STACK:8388608");
    }
}
