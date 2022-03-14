use crate::prelude::*;

#[derive(Debug, Clone, Parser)]
pub struct CoreConfig {
    #[clap(short, long, env = "DISABLE_COLORS")]
    pub disable_colors: bool,
}

pub struct Core;

impl Feature for Core {
    fn init(config: EnvironmentConfig) -> Result<Self> {
        if !config.core.disable_colors {
            color_eyre::install()?;
        }

        Ok(Self)
    }
}
