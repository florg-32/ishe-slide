fn main() {
    let target = std::env::var("TARGET").unwrap();
    if target.contains("android") {
        let ndk = std::env::var("ANDROID_NDK_HOME").unwrap();
        println!(
            "cargo:rustc-link-search={ndk}/toolchains/llvm/prebuilt/linux-x86_64/sysroot/usr/lib/{target}/26"
        );
    }
}
