use std::env;

fn main() {
    let mut target = env::var("TARGET").unwrap();
    target.push_str(" (target_family: ");
    target.push_str(&env::var("CARGO_CFG_TARGET_FAMILY").unwrap());
    target.push_str(", target_os: ");
    target.push_str(&env::var("CARGO_CFG_TARGET_OS").unwrap());
    target.push_str(", target_arch: ");
    target.push_str(&env::var("CARGO_CFG_TARGET_ARCH").unwrap());
    target.push_str(", target_vendor: ");
    target.push_str(&env::var("CARGO_CFG_TARGET_VENDOR").unwrap());
    target.push_str(", target_env: ");
    target.push_str(&env::var("CARGO_CFG_TARGET_ENV").unwrap());
    target.push(')');

    let mut out_file = env::var("OUT_DIR").unwrap();
    out_file.push_str("/target");
    std::fs::write(out_file, target).unwrap();
}
