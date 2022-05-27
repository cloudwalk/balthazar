use serde::Serialize;

/// Returns a struct with the current project build information (Rust, Cargo and Git).
/// 
/// To use this macro, a build.rs file must be configured using [Vergen default configuration](https://docs.rs/vergen/latest/vergen/#buildrs),
/// otherwise the build wil fail because the required environment variables will not be present.
#[macro_export]
macro_rules! generate_build_info {
    () => {
        balthazar::build_info::BuildInfo {
            datetime: env!("VERGEN_BUILD_TIMESTAMP"),
            rust: balthazar::build_info::RustInfo {
                version: env!("VERGEN_RUSTC_SEMVER"),
                platform: env!("VERGEN_CARGO_TARGET_TRIPLE"),
                profile: env!("VERGEN_CARGO_PROFILE"),
                features: env!("VERGEN_CARGO_FEATURES"),
            },
            git: balthazar::build_info::GitInfo {
                branch: env!("VERGEN_GIT_BRANCH"),
                commit_author_email: env!("VERGEN_GIT_COMMIT_AUTHOR_EMAIL")
                commit_author_name: env!("VERGEN_GIT_COMMIT_AUTHOR_NAME")
                commit_datetime: env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
                commit_hash: env!("VERGEN_GIT_SHA"),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildInfo {
    pub datetime: &'static str,
    pub rust: RustInfo,
    pub git: GitInfo,
}

#[derive(Clone, Debug, Serialize)]
pub struct RustInfo {
    pub version: &'static str,
    pub platform: &'static str,
    pub features: &'static str,
    pub profile: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub struct GitInfo {
    pub branch: &'static str,
    pub commit_author_email: &'static str,
    pub commit_author_name: &'static str,
    pub commit_hash: &'static str,
    pub commit_datetime: &'static str,
}
