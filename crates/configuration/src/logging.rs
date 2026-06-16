use tracing_subscriber::{
    EnvFilter, fmt::format::FmtSpan, layer::SubscriberExt, util::SubscriberInitExt,
};

use crate::{ConfigurationError, LogFormat, LoggingConfig};

pub fn init_tracing(config: &LoggingConfig) -> Result<(), ConfigurationError> {
    let env_filter = EnvFilter::try_new(config.level.clone())
        .or_else(|_| EnvFilter::try_new("info"))
        .map_err(|error| ConfigurationError::Invalid(error.to_string()))?;

    let fmt_layer = tracing_subscriber::fmt::layer().with_span_events(FmtSpan::CLOSE);
    let result = match config.format {
        LogFormat::Text => tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer)
            .try_init(),
        LogFormat::Json => tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt_layer.json())
            .try_init(),
    };

    result.map_err(|_| ConfigurationError::TracingInit)
}
