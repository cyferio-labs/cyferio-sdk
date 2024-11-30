use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    println!("cargo::rerun-if-env-changed=SKIP_GUEST_BUILD");
    println!("cargo::rerun-if-env-changed=OUT_DIR");

    if std::env::var("SKIP_GUEST_BUILD").is_ok() {
        return;
    }

    let mut features = vec![];

    if cfg!(feature = "bench") {
        features.push("bench".to_string());
    }

    build_program_with_args(
        "./guest-mock",
        BuildArgs {
            features: features.clone(),
            ..Default::default()
        },
    )
}
