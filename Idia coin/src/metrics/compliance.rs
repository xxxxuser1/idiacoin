use prometheus::{
    IntCounter, IntGauge, Histogram, HistogramOpts,
    register_int_counter, register_int_gauge, register_histogram,
};

lazy_static! {
    // Transaction Metrics
    pub static ref TRANSACTIONS_PROCESSED: IntCounter = register_int_counter!(
        "idia_transactions_processed_total",
        "Total number of transactions processed"
    ).unwrap();

    pub static ref TRANSACTIONS_REJECTED: IntCounter = register_int_counter!(
        "idia_transactions_rejected_total",
        "Total number of transactions rejected"
    ).unwrap();

    pub static ref TRANSACTION_SIZE: Histogram = register_histogram!(
        "idia_transaction_size_bytes",
        "Transaction size distribution in bytes",
        vec![100.0, 500.0, 1000.0, 5000.0, 10000.0]
    ).unwrap();

    // Privacy Feature Metrics
    pub static ref RING_SIGNATURE_SIZE: Histogram = register_histogram!(
        "idia_ring_signature_size",
        "Ring signature size distribution",
        vec![5.0, 7.0, 11.0, 15.0, 21.0]
    ).unwrap();

    pub static ref STEALTH_ADDRESS_USAGE: IntCounter = register_int_counter!(
        "idia_stealth_address_usage_total",
        "Number of stealth addresses used"
    ).unwrap();

    // Compliance Metrics
    pub static ref COMPLIANCE_CHECKS_TOTAL: IntCounter = register_int_counter!(
        "idia_compliance_checks_total",
        "Total number of compliance checks performed"
    ).unwrap();

    pub static ref COMPLIANCE_CHECK_FAILURES: IntCounter = register_int_counter!(
        "idia_compliance_check_failures_total",
        "Number of failed compliance checks"
    ).unwrap();

    pub static ref HIGH_RISK_TRANSACTIONS: IntGauge = register_int_gauge!(
        "idia_high_risk_transactions_current",
        "Current number of high-risk transactions under review"
    ).unwrap();

    // Regulatory Reporting Metrics
    pub static ref VIEW_KEY_REQUESTS: IntCounter = register_int_counter!(
        "idia_view_key_requests_total",
        "Total number of view key requests"
    ).unwrap();

    pub static ref REGULATORY_REPORTS_GENERATED: IntCounter = register_int_counter!(
        "idia_regulatory_reports_generated_total",
        "Total number of regulatory reports generated"
    ).unwrap();

    // Network Privacy Metrics
    pub static ref TOR_CONNECTIONS: IntGauge = register_int_gauge!(
        "idia_tor_connections_current",
        "Current number of Tor connections"
    ).unwrap();

    pub static ref DANDELION_STEM_PHASE_TRANSACTIONS: IntGauge = register_int_gauge!(
        "idia_dandelion_stem_phase_transactions",
        "Current number of transactions in Dandelion++ stem phase"
    ).unwrap();
}

pub fn record_transaction_metrics(tx: &Transaction) {
    TRANSACTIONS_PROCESSED.inc();
    TRANSACTION_SIZE.observe(tx.serialized_size() as f64);
    RING_SIGNATURE_SIZE.observe(tx.ring_size() as f64);
    
    if tx.uses_stealth_address() {
        STEALTH_ADDRESS_USAGE.inc();
    }
}

pub fn record_compliance_check(check: &ComplianceCheck) {
    COMPLIANCE_CHECKS_TOTAL.inc();
    
    match check.result {
        CheckResult::Fail(_) => {
            COMPLIANCE_CHECK_FAILURES.inc();
        },
        CheckResult::Warning(_) => {
            HIGH_RISK_TRANSACTIONS.inc();
        },
        _ => {}
    }
}

pub fn record_regulatory_activity(activity: RegulatoryActivity) {
    match activity {
        RegulatoryActivity::ViewKeyRequest => VIEW_KEY_REQUESTS.inc(),
        RegulatoryActivity::ReportGeneration => REGULATORY_REPORTS_GENERATED.inc(),
    }
}

pub fn update_network_metrics(metrics: NetworkMetrics) {
    TOR_CONNECTIONS.set(metrics.tor_connections as i64);
    DANDELION_STEM_PHASE_TRANSACTIONS.set(metrics.dandelion_stem_tx as i64);
}