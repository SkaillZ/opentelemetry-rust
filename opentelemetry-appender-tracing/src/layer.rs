use opentelemetry::logs::{LogRecord, Logger, LoggerProvider, Severity};
use std::borrow::Cow;
use tracing_core::Level;
use tracing_subscriber::Layer;

const INSTRUMENTATION_LIBRARY_NAME: &str = "opentelemetry-appender-tracing";

/// Visitor to record the fields from the event record.
struct EventVisitor<'a> {
    log_record: &'a mut LogRecord,
}

impl<'a> tracing::field::Visit for EventVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.log_record.body = Some(format!("{value:?}").into());
        } else if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), format!("{value:?}").into()));
        } else {
            let vec = vec![(field.name().into(), format!("{value:?}").into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_str(&mut self, field: &tracing_core::Field, value: &str) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.to_owned().into()));
        } else {
            let vec = vec![(field.name().into(), value.to_owned().into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_bool(&mut self, field: &tracing_core::Field, value: bool) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.into()));
        } else {
            let vec = vec![(field.name().into(), value.into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.into()));
        } else {
            let vec = vec![(field.name().into(), value.into())];
            self.log_record.attributes = Some(vec);
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        if let Some(ref mut vec) = self.log_record.attributes {
            vec.push((field.name().into(), value.into()));
        } else {
            let vec = vec![(field.name().into(), value.into())];
            self.log_record.attributes = Some(vec);
        }
    }

    // TODO: Remaining field types from AnyValue : Bytes, ListAny, Boolean
}

pub struct OpenTelemetryTracingBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    logger: L,
    _phantom: std::marker::PhantomData<P>, // P is not used.
}

impl<P, L> OpenTelemetryTracingBridge<P, L>
where
    P: LoggerProvider<Logger = L> + Send + Sync,
    L: Logger + Send + Sync,
{
    pub fn new(provider: &P) -> Self {
        OpenTelemetryTracingBridge {
            logger: provider.versioned_logger(
                INSTRUMENTATION_LIBRARY_NAME,
                Some(Cow::Borrowed(env!("CARGO_PKG_VERSION"))),
                None,
                None,
            ),
            _phantom: Default::default(),
        }
    }
}

impl<S, P, L> Layer<S> for OpenTelemetryTracingBridge<P, L>
where
    S: tracing::Subscriber,
    P: LoggerProvider<Logger = L> + Send + Sync + 'static,
    L: Logger + Send + Sync + 'static,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        let meta = event.metadata();
        let mut log_record: LogRecord = LogRecord::default();
        log_record.severity_number = Some(severity_of_level(meta.level()));
        log_record.severity_text = Some(meta.level().to_string().into());

        // add the `name` metadata to attributes
        // TBD - Propose this to be part of log_record metadata.
        let vec = vec![("name", meta.name())];
        log_record.attributes = Some(vec.into_iter().map(|(k, v)| (k.into(), v.into())).collect());

        // Not populating ObservedTimestamp, instead relying on OpenTelemetry
        // API to populate it with current time.

        let mut visitor = EventVisitor {
            log_record: &mut log_record,
        };
        event.record(&mut visitor);
        self.logger.emit(log_record);
    }

    #[cfg(feature = "logs_level_enabled")]
    fn event_enabled(
        &self,
        _event: &tracing_core::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) -> bool {
        let severity = severity_of_level(_event.metadata().level());
        self.logger
            .event_enabled(severity, _event.metadata().target())
    }
}

const fn severity_of_level(level: &Level) -> Severity {
    match *level {
        Level::TRACE => Severity::Trace,
        Level::DEBUG => Severity::Debug,
        Level::INFO => Severity::Info,
        Level::WARN => Severity::Warn,
        Level::ERROR => Severity::Error,
    }
}
