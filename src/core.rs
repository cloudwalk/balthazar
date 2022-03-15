use crate::prelude::*;

#[derive(Debug, Clone, Parser)]
pub struct CoreConfig {
    #[clap(short, long, env = "NO_COLORS")]
    pub no_colors: bool,
}

pub struct Core;

impl Feature for Core {
    fn init(config: EnvironmentConfig) -> Result<Self> {
        if !config.core.no_colors {
            color_eyre::install()?;
        }

        Ok(Self)
    }
}
