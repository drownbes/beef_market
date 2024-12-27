use once_cell::sync::Lazy;
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    EnvFilter, Registry,
};

pub static TRACING: Lazy<()> = Lazy::new(|| {
    let fmt_layer = fmt::layer()
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);

    let filter_layer = EnvFilter::from_default_env();
    LogTracer::init().unwrap();

    let subscriber = Registry::default().with(fmt_layer).with(filter_layer);

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
});

pub fn init_tracing() {
    Lazy::force(&TRACING);
}
