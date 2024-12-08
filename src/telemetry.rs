use opentelemetry::{ 
    metrics::Result as OtelResult,
    KeyValue,
    trace::{TracerProvider, TraceError},
};
use opentelemetry_sdk::{
    metrics::{
        MeterProvider, ManualReader, reader::{AggregationSelector, TemporalitySelector, MetricReader},
        data::{Temporality, ResourceMetrics},
        InstrumentKind, Pipeline, Aggregation,
    },
    trace::TracerProvider as SdkTracerProvider,
    export::trace::SpanData,
};

use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use opentelemetry::global;

/// Manages telemetry for the application, including metrics and tracing
pub struct TelemetryManager {
    tracer_provider: SdkTracerProvider,
}

/// Console exporter for metrics and spans
#[derive(Debug, Default)]
struct ConsoleExporter;

impl AggregationSelector for ConsoleExporter {
    fn aggregation<'a>(&'a self, _kind: InstrumentKind) -> Box<dyn Iterator<Item = Aggregation> + 'a> {
        Box::new(std::iter::once(Aggregation::Sum()))
    }
}

impl TemporalitySelector for ConsoleExporter {
    fn temporality(&self, _kind: InstrumentKind) -> Temporality {
        Temporality::Cumulative
    }
}

impl MetricReader for ConsoleExporter {
    fn register_pipeline(&self, _pipeline: std::sync::Weak<Pipeline>) {
        println!("Pipeline registered");
    }

    fn collect(&self, metrics: &mut ResourceMetrics) -> OtelResult<()> {
        println!("Metrics: {:?}", metrics);
        Ok(())
    }

    fn force_flush(&self) -> OtelResult<()> {
        Ok(())
    }

    fn shutdown(&self) -> OtelResult<()> {
        Ok(())
    }
}

impl opentelemetry_sdk::export::trace::SpanExporter for ConsoleExporter {
    fn export(&mut self, spans: Vec<SpanData>) -> Result<(), TraceError> {
        println!("Spans: {:?}", spans);
        Ok(())
    }
}

impl TelemetryManager {
    /// Initialize the telemetry system with metrics and tracing
    pub async fn init() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Set up metrics with manual reader
        let reader = ManualReader::builder()
            .with_temporality_selector(ConsoleExporter::default())
            .with_aggregation_selector(ConsoleExporter::default())
            .build();

        let meter_provider = MeterProvider::builder()
            .with_reader(reader)
            .build();

        global::set_meter_provider(meter_provider);

        // Set up tracing
        let tracer_provider = SdkTracerProvider::builder()
            .with_simple_exporter(ConsoleExporter::default())
            .build();

        global::set_tracer_provider(tracer_provider.clone());
        let tracer = tracer_provider.versioned_tracer(
            "omnipro_db",
            Some(env!("CARGO_PKG_VERSION")),
            Some(env!("CARGO_PKG_NAME")),
            None,
        );

        let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::from_default_env())
            .with(telemetry);

        tracing::subscriber::set_global_default(subscriber)?;

        Ok(Self { tracer_provider })
    }

    /// Record a metric with the given name, value, and attributes
    pub fn record_metric(&self, name: String, value: f64, attributes: Vec<(String, String)>) {
        let meter = global::meter("omnipro_db");
        let counter = meter.f64_counter(name).init();

        let attrs: Vec<KeyValue> = attributes
            .into_iter()
            .map(|(k, v)| KeyValue::new(k, v))
            .collect();

        counter.add(value, &attrs);
    }
}

impl Drop for TelemetryManager {
    fn drop(&mut self) {
        global::shutdown_tracer_provider();
    }
}
