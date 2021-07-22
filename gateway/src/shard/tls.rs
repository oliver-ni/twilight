//! TLS manager to reuse connections between shards.

#[cfg(feature = "rustls")]
use std::sync::Arc;
use std::{
    error::Error,
    fmt::{Display, Formatter, Result as FmtResult},
};

#[cfg(all(feature = "native", not(feature = "rustls")))]
use native_tls::TlsConnector as NativeTlsConnector;
#[cfg(feature = "rustls")]
use rustls_tls::ClientConfig;
use tokio_tungstenite::Connector;
use url::Url;

#[cfg(all(feature = "native", not(feature = "rustls")))]
pub type TlsConnector = NativeTlsConnector;
#[cfg(feature = "rustls")]
pub type TlsConnector = Arc<ClientConfig>;

#[derive(Debug)]
pub struct TlsError {
    kind: TlsErrorType,
    source: Option<Box<dyn Error + Send + Sync>>,
}

#[allow(dead_code)]
impl TlsError {
    /// Immutable reference to the type of error that occurred.
    #[must_use = "retrieving the type has no effect if left unused"]
    pub const fn kind(&self) -> &TlsErrorType {
        &self.kind
    }

    /// Consume the error, returning the source error if there is any.
    #[must_use = "consuming the error and retrieving the source has no effect if left unused"]
    pub fn into_source(self) -> Option<Box<dyn Error + Send + Sync>> {
        self.source
    }

    /// Consume the error, returning the owned error type and the source error.
    #[must_use = "consuming the error into its parts has no effect if left unused"]
    pub fn into_parts(self) -> (TlsErrorType, Option<Box<dyn Error + Send + Sync>>) {
        (self.kind, self.source)
    }
}

impl Display for TlsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.kind {
            #[cfg(all(feature = "native", not(feature = "rustls")))]
            TlsErrorType::NativeTls => {
                f.write_str("construction of the nativetls connector failed")
            }
            #[cfg(feature = "rustls-native-roots")]
            TlsErrorType::NativeCerts => f.write_str("could not load native certificates"),
            TlsErrorType::NoDomain => f.write_str("URL provided by discord have no domain part"),
        }
    }
}

impl Error for TlsError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| &**source as &(dyn Error + 'static))
    }
}

/// Type of [`ClusterCommandError`] that occurred.
#[derive(Debug)]
#[non_exhaustive]
pub enum TlsErrorType {
    /// Construction of the nativetls connector failed.
    #[cfg(all(feature = "native", not(feature = "rustls")))]
    NativeTls,
    /// Could not load native certificates.
    #[cfg(feature = "rustls-native-roots")]
    NativeCerts,
    /// URL provided by discord have no domain part.
    NoDomain,
}

#[derive(Clone)]
#[cfg_attr(all(feature = "native", not(feature = "rustls")), derive(Debug))]
pub struct TlsContainer {
    tls: TlsConnector,
}

#[cfg(feature = "rustls")]
impl std::fmt::Debug for TlsContainer {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.debug_struct("TlsContainer").finish()
    }
}

impl TlsContainer {
    #[cfg(all(feature = "native", not(feature = "rustls")))]
    pub fn new() -> Result<Self, TlsError> {
        let native_connector = TlsConnector::new().map_err(|err| TlsError {
            kind: TlsErrorType::NativeTls,
            source: Some(Box::new(err)),
        })?;

        Ok(TlsContainer {
            tls: native_connector,
        })
    }

    #[cfg(feature = "rustls")]
    pub fn new() -> Result<Self, TlsError> {
        let mut config = ClientConfig::new();

        #[cfg(feature = "rustls-native-roots")]
        {
            let native_certs =
                rustls_native_certs::load_native_certs().map_err(|(_, err)| TlsError {
                    kind: TlsErrorType::NativeCerts,
                    source: Some(Box::new(err)),
                })?;

            config.root_store = native_certs;
        }

        #[cfg(feature = "rustls-webpki-roots")]
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        Ok(TlsContainer {
            tls: Arc::new(config),
        })
    }

    pub fn tls_domain(&self, url: &Url) -> Result<(String, Connector), TlsError> {
        let domain = url.domain().ok_or(TlsError {
            kind: TlsErrorType::NoDomain,
            source: None,
        })?;

        let mut address = String::with_capacity(domain.len() + 4);
        address.push_str(domain);
        address.push_str(":443");

        #[cfg(all(feature = "native", not(feature = "rustls")))]
        return Ok((address, Connector::NativeTls(self.tls.clone())));

        #[cfg(feature = "rustls")]
        return Ok((address, Connector::Rustls(Arc::clone(&self.tls))));
    }
}

#[cfg(test)]
static_assertions::assert_impl_all!(TlsContainer: std::fmt::Debug, Clone, Send, Sync);
