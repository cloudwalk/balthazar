use serde::Serialize;

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
    pub commit_hash: &'static str,
    pub commit_datetime: &'static str,
}