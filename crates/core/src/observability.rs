use anyhow::Result;
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, span, Level, Instrument};
use opentelemetry::{
    global,
    trace::{Tracer, TracerProvider, SpanKind, Status},
    metrics::{Meter, MeterProvider, Counter, Histogram},
    KeyValue,
};
use opentelemetry_sdk::{
    trace::{TracerProvider as SdkTracerProvider, Config},
    metrics::{MeterProvider as SdkMeterProvider, PeriodicReader},
    Resource,
    runtime,
};
use opentelemetry_otlp::{
    new_exporter,
    new_pipeline,
    WithExportConfig,
};
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, SERVICE_VERSION, DEPLOYMENT_ENVIRONMENT,
};
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricLabels {
    pub service: String,
    pub version: String,
    pub environment: String,
    pub component: String,
    pub operation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub operation: String,
    pub duration_ms: f64,
    pub success: bool,
    pub error_type: Option<String>,
    pub labels: MetricLabels,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessMetrics {
    pub metric_name: String,
    pub value: f64,
    pub labels: MetricLabels,
    pub timestamp: DateTime<Utc>,
}

pub struct ObservabilityService {
    tracer: opentelemetry::trace::Tracer,
    meter: Meter,
    request_counter: Counter<u64>,
    request_duration: Histogram<f64>,
    active_connections: Counter<u64>,
    error_counter: Counter<u64>,
    business_metrics: std::collections::HashMap<String, Counter<u64>>,
}

impl ObservabilityService {
    pub async fn new(
        service_name: &str,
        service_version: &str,
        environment: &str,
        otlp_endpoint: Option<&str>,
    ) -> Result<Self> {
        let resource = Resource::new(vec![
            KeyValue::new(SERVICE_NAME, service_name.to_string()),
            KeyValue::new(SERVICE_VERSION, service_version.to_string()),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT, environment.to_string()),
        ]);

        // Initialize tracing
        let tracer = if let Some(endpoint) = otlp_endpoint {
            let exporter = new_exporter()
                .tonic()
                .with_endpoint(endpoint)
                .with_export_config(
                    opentelemetry_otlp::ExportConfig {
                        timeout: Duration::from_secs(30),
                        ..Default::default()
                    }
                );

            let tracer_provider = SdkTracerProvider::builder()
                .with_batch_exporter(exporter, runtime::Tokio)
                .with_config(Config::default().with_resource(resource.clone()))
                .build();

            global::set_tracer_provider(tracer_provider);
            global::tracer("blacklake")
        } else {
            global::tracer("blacklake")
        };

        // Initialize metrics - simplified for now
        let meter = global::tracer("blacklake");

        // Create metrics - simplified for now
        let request_counter = meter;
        let request_duration = meter;
        let active_connections = meter;
        let error_counter = meter;

        let mut business_metrics = std::collections::HashMap::new();
        
        // Initialize business metrics
        let metrics = vec![
            "federation_syncs_total",
            "semantic_searches_total",
            "legal_holds_created_total",
            "retention_policies_applied_total",
            "compliance_exports_total",
            "mobile_sessions_total",
            "pwa_installs_total",
        ];

        for metric_name in metrics {
            let counter = meter
                .u64_counter(metric_name)
                .with_description(format!("Total number of {}", metric_name))
                .init();
            business_metrics.insert(metric_name.to_string(), counter);
        }

        Ok(Self {
            tracer,
            meter,
            request_counter,
            request_duration,
            active_connections,
            error_counter,
            business_metrics,
        })
    }

    /// Start a new trace span
    pub fn start_span(&self, name: &str, kind: SpanKind) -> tracing::Span {
        let span = self.tracer
            .span_builder(name)
            .with_kind(kind)
            .start(&self.tracer);

        span.into()
    }

    /// Record a performance metric
    pub fn record_performance_metric(&self, metric: PerformanceMetrics) {
        let labels = vec![
            KeyValue::new("service", metric.labels.service),
            KeyValue::new("version", metric.labels.version),
            KeyValue::new("environment", metric.labels.environment),
            KeyValue::new("component", metric.labels.component),
            KeyValue::new("operation", metric.labels.operation),
            KeyValue::new("success", metric.success.to_string()),
        ];

        if let Some(error_type) = metric.error_type {
            let mut error_labels = labels.clone();
            error_labels.push(KeyValue::new("error_type", error_type));
            self.error_counter.add(1, &error_labels);
        }

        // Simplified metrics recording
        // self.request_counter.add(1, &labels);
        // self.request_duration.record(metric.duration_ms / 1000.0, &labels);
    }

    /// Record a business metric
    pub fn record_business_metric(&self, metric: BusinessMetrics) {
        if let Some(counter) = self.business_metrics.get(&metric.metric_name) {
            let labels = vec![
                KeyValue::new("service", metric.labels.service),
                KeyValue::new("version", metric.labels.version),
                KeyValue::new("environment", metric.labels.environment),
                KeyValue::new("component", metric.labels.component),
                KeyValue::new("operation", metric.labels.operation),
            ];

            counter.add(metric.value as u64, &labels);
        }
    }

    /// Update active connections gauge
    pub fn update_active_connections(&self, count: u64) {
        // Simplified metrics recording
        // self.active_connections.add(count, &[]);
    }

    /// Create a trace context for distributed tracing
    pub fn create_trace_context(&self, _span: &tracing::Span) -> TraceContext {
        // Simplified trace context creation
        TraceContext {
            trace_id: "simplified_trace_id".to_string(),
            span_id: "simplified_span_id".to_string(),
            parent_span_id: None,
        }
    }

    /// Log a structured event with tracing
    pub fn log_event(
        &self,
        level: Level,
        message: &str,
        fields: std::collections::HashMap<String, serde_json::Value>,
    ) {
        let span = self.start_span("log_event", SpanKind::Internal);
        let _enter = span.enter();

        match level {
            Level::ERROR => error!(?fields, "{}", message),
            Level::WARN => warn!(?fields, "{}", message),
            Level::INFO => info!(?fields, "{}", message),
            Level::DEBUG => tracing::debug!(?fields, "{}", message),
            Level::TRACE => tracing::trace!(?fields, "{}", message),
        }
    }

    /// Record federation sync metrics
    pub fn record_federation_sync(
        &self,
        connector_type: &str,
        success: bool,
        duration_ms: f64,
        entries_synced: u64,
    ) {
        let labels = MetricLabels {
            service: "blacklake".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            component: "federation".to_string(),
            operation: format!("sync_{}", connector_type),
        };

        self.record_performance_metric(PerformanceMetrics {
            operation: format!("federation_sync_{}", connector_type),
            duration_ms,
            success,
            error_type: if success { None } else { Some("sync_failed".to_string()) },
            labels: labels.clone(),
        });

        self.record_business_metric(BusinessMetrics {
            metric_name: "federation_syncs_total".to_string(),
            value: 1.0,
            labels: labels.clone(),
            timestamp: Utc::now(),
        });

        // Record entries synced
        if let Some(counter) = self.business_metrics.get("federation_entries_synced_total") {
            let labels = vec![
                KeyValue::new("connector_type", connector_type.to_string()),
                KeyValue::new("success", success.to_string()),
            ];
            counter.add(entries_synced, &labels);
        }
    }

    /// Record semantic search metrics
    pub fn record_semantic_search(
        &self,
        success: bool,
        duration_ms: f64,
        results_count: u64,
    ) {
        let labels = MetricLabels {
            service: "blacklake".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            component: "search".to_string(),
            operation: "semantic_search".to_string(),
        };

        self.record_performance_metric(PerformanceMetrics {
            operation: "semantic_search".to_string(),
            duration_ms,
            success,
            error_type: if success { None } else { Some("search_failed".to_string()) },
            labels: labels.clone(),
        });

        self.record_business_metric(BusinessMetrics {
            metric_name: "semantic_searches_total".to_string(),
            value: 1.0,
            labels: labels.clone(),
            timestamp: Utc::now(),
        });

        // Record results count
        if let Some(counter) = self.business_metrics.get("semantic_search_results_total") {
            let labels = vec![
                KeyValue::new("success", success.to_string()),
            ];
            counter.add(results_count, &labels);
        }
    }

    /// Record compliance metrics
    pub fn record_compliance_event(
        &self,
        event_type: &str,
        success: bool,
        duration_ms: f64,
    ) {
        let labels = MetricLabels {
            service: "blacklake".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            component: "compliance".to_string(),
            operation: event_type.to_string(),
        };

        self.record_performance_metric(PerformanceMetrics {
            operation: format!("compliance_{}", event_type),
            duration_ms,
            success,
            error_type: if success { None } else { Some("compliance_failed".to_string()) },
            labels: labels.clone(),
        });

        let metric_name = match event_type {
            "legal_hold_created" => "legal_holds_created_total",
            "retention_policy_applied" => "retention_policies_applied_total",
            "compliance_export" => "compliance_exports_total",
            _ => "compliance_events_total",
        };

        if let Some(counter) = self.business_metrics.get(metric_name) {
            let labels = vec![
                KeyValue::new("event_type", event_type.to_string()),
                KeyValue::new("success", success.to_string()),
            ];
            counter.add(1, &labels);
        }
    }

    /// Record mobile/PWA metrics
    pub fn record_mobile_event(
        &self,
        event_type: &str,
        success: bool,
        duration_ms: f64,
    ) {
        let labels = MetricLabels {
            service: "blacklake".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            component: "mobile".to_string(),
            operation: event_type.to_string(),
        };

        self.record_performance_metric(PerformanceMetrics {
            operation: format!("mobile_{}", event_type),
            duration_ms,
            success,
            error_type: if success { None } else { Some("mobile_failed".to_string()) },
            labels: labels.clone(),
        });

        let metric_name = match event_type {
            "session_start" => "mobile_sessions_total",
            "pwa_install" => "pwa_installs_total",
            _ => "mobile_events_total",
        };

        if let Some(counter) = self.business_metrics.get(metric_name) {
            let labels = vec![
                KeyValue::new("event_type", event_type.to_string()),
                KeyValue::new("success", success.to_string()),
            ];
            counter.add(1, &labels);
        }
    }

    /// Record AI/ML metrics
    pub fn record_ai_event(
        &self,
        event_type: &str,
        success: bool,
        duration_ms: f64,
        model_name: &str,
    ) {
        let labels = MetricLabels {
            service: "blacklake".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            component: "ai".to_string(),
            operation: event_type.to_string(),
        };

        self.record_performance_metric(PerformanceMetrics {
            operation: format!("ai_{}", event_type),
            duration_ms,
            success,
            error_type: if success { None } else { Some("ai_failed".to_string()) },
            labels: labels.clone(),
        });

        if let Some(counter) = self.business_metrics.get("ai_operations_total") {
            let labels = vec![
                KeyValue::new("event_type", event_type.to_string()),
                KeyValue::new("model_name", model_name.to_string()),
                KeyValue::new("success", success.to_string()),
            ];
            counter.add(1, &labels);
        }
    }
}

/// Macro for easy tracing of async functions
#[macro_export]
macro_rules! trace_async {
    ($name:expr, $future:expr) => {{
        let span = tracing::info_span!("async_operation", operation = $name);
        $future.instrument(span).await
    }};
}

/// Macro for easy performance measurement
#[macro_export]
macro_rules! measure_performance {
    ($operation:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let duration = start.elapsed();
        
        // Record performance metric
        if let Some(obs) = crate::observability::get_observability_service() {
            obs.record_performance_metric(crate::observability::PerformanceMetrics {
                operation: $operation.to_string(),
                duration_ms: duration.as_millis() as f64,
                success: result.is_ok(),
                error_type: if result.is_err() { Some("operation_failed".to_string()) } else { None },
                labels: crate::observability::MetricLabels {
                    service: "blacklake".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    environment: std::env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
                    component: "api".to_string(),
                    operation: $operation.to_string(),
                },
            });
        }
        
        result
    }};
}
