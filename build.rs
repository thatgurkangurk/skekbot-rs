#[allow(clippy::result_large_err)]
fn main() -> Result<(), nlprule_build::Error> {
    // Only re-run if this script changes
    println!("cargo:rerun-if-changed=build.rs");

    // This automatically fetches, unzips, caches, and validates
    // the English tokenizer and grammar rules into your OUT_DIR.
    nlprule_build::BinaryBuilder::new(
        &["en"],
        #[allow(clippy::expect_used)]
        std::env::var("OUT_DIR").expect("OUT_DIR is set when build.rs is running"),
    )
    .build()?
    .validate()
}
