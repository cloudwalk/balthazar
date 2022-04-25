use balthazar::*;

#[derive(clap::Parser, Debug, Clone)]
struct AppConfig {}

#[balthazar::main]
async fn main() -> Result<()> {
    let _ = Config::<AppConfig>::init("basic_example").await?;

    Ok(())
}
