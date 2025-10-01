// BlackLake Federation Connectors
// Week 8: Federation across data sources

pub mod traits;
pub mod s3;
pub mod postgres;
pub mod ckan;
pub mod manager;

pub use traits::{Connector, ConnectorRegistry, ConnectorType, ConnectorConfig, ConnectorStatus, ConnectorError};
pub use manager::ConnectorManager;
