fn main() {
    println!("cargo:rerun-if-env-changed=CARGO_CFG_TARGET_ARCH");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("arm") {
        println!("cargo:rustc-cfg=feature=\"signer\"");
    }
}
