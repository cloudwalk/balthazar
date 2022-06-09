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
                version: option_env!("VERGEN_RUSTC_SEMVER").unwrap_or(""),
                platform: option_env!("VERGEN_CARGO_TARGET_TRIPLE").unwrap_or(""),
                profile: option_env!("VERGEN_CARGO_PROFILE").unwrap_or(""),
                features: option_env!("VERGEN_CARGO_FEATURES").unwrap_or(""),
            },
            git: balthazar::build_info::GitInfo {
                branch: option_env!("VERGEN_GIT_BRANCH").unwrap_or(""),
                commit_author_email: option_env!("VERGEN_GIT_COMMIT_AUTHOR_EMAIL").unwrap_or(""),
                commit_author_name: option_env!("VERGEN_GIT_COMMIT_AUTHOR_NAME").unwrap_or(""),
                commit_datetime: option_env!("VERGEN_GIT_COMMIT_TIMESTAMP").unwrap_or(""),
                commit_hash: option_env!("VERGEN_GIT_SHA").unwrap_or(""),
                commit_message: option_env!("VERGEN_GIT_COMMIT_MESSAGE").unwrap_or(""),
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
    pub commit_message: &'static str,
}
