use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().with_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(tracing::Level::INFO.into())
                    .from_env_lossy(),
            ),
        )
        .with(tracing_error::ErrorLayer::default())
        .init();

    Ok(())
}
