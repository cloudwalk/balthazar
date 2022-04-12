use crate::{EnvironmentConfig, Feature, Parser, Result};

#[derive(Debug, Clone, Parser)]
pub struct CoreConfig {
    #[clap(short, long, env = "NO_COLOR")]
    pub no_color: bool,
}

pub struct Core;

impl Feature for Core {
    fn init(_service_name: String, config: EnvironmentConfig) -> Result<Self> {
        if !config.core.no_color {
            color_eyre::install()?;
        }

        Ok(Self)
    }
}
