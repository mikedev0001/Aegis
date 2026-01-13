fn main() {
    // Ensure KVM is available on Linux
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-cfg=kvm");
    }
    
    // Link system libraries
    if cfg!(target_os = "linux") {
        println!("cargo:rustc-link-lib=util");
        println!("cargo:rustc-link-lib=rt");
    }
}