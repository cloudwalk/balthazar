use crate::*;

#[derive(Debug, Clone, Parser)]
pub struct CoreConfig {
    #[clap(short, long, env = "NO_COLOR")]
    pub no_color: bool,
}

pub struct Core;

#[crate::async_trait]
impl Feature for Core {
    async fn init(_service_name: &str, config: EnvironmentConfig) -> Result<Self> {
        if !config.core.no_color {
            color_eyre::install()?;
        }

        Ok(Self)
    }
}
