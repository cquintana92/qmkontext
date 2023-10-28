#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct IsoTime;
impl tracing_subscriber::fmt::time::FormatTime for IsoTime {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = qmkontext::chrono::Local::now().naive_utc();
        let formatted = now.format("%Y-%m-%d %H:%M:%S%.3f");
        write!(w, "[{}] ", formatted)
    }
}

pub fn setup_logging(log_level: &str) {
    let log_level_lower = log_level.to_lowercase();
    let rust_log_default_value = if log_level_lower == "trace" {
        "qmkontext=trace,qmkontext_cli=trace".to_string()
    } else {
        format!("qmkontext={0},qmkontext_cli={0}", log_level)
    };

    tracing_log::env_logger::init();
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_timer(IsoTime)
        .with_env_filter(rust_log_default_value)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
}
