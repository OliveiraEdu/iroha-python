//! iroha2-python sys library with all classes (wrapped rust structures) with methods

// Allow panic because of bad and unsafe pyo3
#![allow(
    clippy::panic,
    clippy::needless_pass_by_value,
    clippy::used_underscore_binding,
    clippy::multiple_inherent_impl
)]

use core::ops::{Deref, DerefMut};
use std::collections::HashMap;
use std::str::FromStr;

use color_eyre::eyre;
use iroha_client::client;
use iroha_config::client::Configuration;
use iroha_crypto::{Algorithm, Hash, KeyGenConfiguration, SignatureOf};
use iroha_crypto::{PrivateKey, PublicKey};
use iroha_data_model::prelude::*;
use iroha_version::scale::EncodeVersioned;
use parity_scale_codec::{Decode, Encode};
use pyo3::class::iter::IterNextOutput;
use pyo3::prelude::*;

use crate::python::*;

mod python;
mod types;

#[pymethods]
impl KeyPair {
    /// Generates new key
    /// # Errors
    #[new]
    pub fn generate() -> PyResult<Self> {
        iroha_crypto::KeyPair::generate()
            .map_err(to_py_err)
            .map(Into::into)
    }

    /// Generate keypair from hex-encoded private key and algorithm
    /// `algorithm` defaults to ed25519
    /// # Errors
    #[staticmethod]
    pub fn from_private(pk: String, algorithm: Option<String>) -> PyResult<Self> {
        let algorithm = Algorithm::from_str(algorithm.as_deref().unwrap_or(iroha_crypto::ED_25519))
            .map_err(to_py_err)?;
        let pk = PrivateKey::from_hex(algorithm, &pk).map_err(to_py_err)?;
        let cfg = KeyGenConfiguration::default()
            .with_algorithm(algorithm)
            .use_private_key(pk);
        iroha_crypto::KeyPair::generate_with_configuration(cfg)
            .map_err(to_py_err)
            .map(Into::into)
    }

    /// Create keypair with some seed
    /// # Errors
    #[staticmethod]
    pub fn with_seed(seed: Vec<u8>) -> PyResult<Self> {
        let cfg = KeyGenConfiguration::default().use_seed(seed);
        iroha_crypto::KeyPair::generate_with_configuration(cfg)
            .map_err(to_py_err)
            .map(Into::into)
    }

    /// Gets public key
    #[getter]
    pub fn public(&self) -> ToPy<PublicKey> {
        ToPy(self.public_key().clone())
    }

    /// Gets private key
    #[getter]
    pub fn private(&self) -> ToPy<PrivateKey> {
        ToPy(self.private_key().clone())
    }

    /// Sign arbitrary `bytes`
    /// `bytes` should not be prehashed
    pub fn sign(&self, bytes: Vec<u8>) -> PyResult<Vec<u8>> {
        SignatureOf::new(self.deref().clone(), &bytes)
            .map_err(to_py_err)
            .map(|sig| sig.payload().to_owned())
            .map(Into::into)
    }
}

/// Hash bytes
#[pyfunction]
pub fn hash(bytes: Vec<u8>) -> ToPy<Hash> {
    ToPy(Hash::new(&bytes))
}

#[pymethods]
impl Client {
    /// Creates new client
    #[new]
    pub fn new(cfg: ToPy<Configuration>) -> PyResult<Self> {
        client::Client::new(&cfg).map_err(to_py_err).map(Self::from)
    }

    /// Creates new client with specified headers
    ///
    /// # Errors
    /// - If configuration isn't valid
    #[staticmethod]
    pub fn with_headers(
        cfg: ToPy<Configuration>,
        headers: HashMap<String, String>,
    ) -> PyResult<Self> {
        client::Client::with_headers(&cfg, headers)
            .map_err(to_py_err)
            .map(Self::from)
    }

    /// Queries peer
    /// # Errors
    /// Can fail if there is no access to peer
    pub fn request(&mut self, query: ToPy<QueryBox>) -> PyResult<ToPy<Value>> {
        self.deref_mut()
            .request(query.into_inner())
            .map_err(to_py_err)
            .map(ToPy)
    }

    /// Get transaction body
    /// # Errors
    pub fn tx_body(
        &mut self,
        isi: Vec<ToPy<Instruction>>,
        metadata: ToPy<UnlimitedMetadata>,
    ) -> PyResult<Vec<u8>> {
        let isi = isi.into_iter().map(ToPy::into_inner).into();
        self.build_transaction(isi, metadata.into_inner())
            .map(VersionedSignedTransaction::from)
            .map_err(to_py_err)
            .map(|tx| tx.encode_versioned())
    }

    /// Sends transaction to peer
    /// # Errors
    /// Can fail if there is no access to peer
    pub fn submit_all_with_metadata(
        &mut self,
        isi: Vec<ToPy<Instruction>>,
        metadata: ToPy<UnlimitedMetadata>,
    ) -> PyResult<ToPy<Hash>> {
        let isi = isi.into_iter().map(ToPy::into_inner);
        self.deref_mut()
            .submit_all_with_metadata(isi, metadata.into_inner())
            .map(|h| *h)
            .map_err(to_py_err)
            .map(ToPy)
    }

    /// Sends transaction to peer and waits till its finalization
    /// # Errors
    /// Can fail if there is no access to peer
    pub fn submit_all_blocking_with_metadata(
        &mut self,
        isi: Vec<ToPy<Instruction>>,
        metadata: ToPy<UnlimitedMetadata>,
    ) -> PyResult<ToPy<Hash>> {
        let isi = isi.into_iter().map(ToPy::into_inner);
        self.deref_mut()
            .submit_all_blocking_with_metadata(isi, metadata.into_inner())
            .map(|h| *h)
            .map_err(to_py_err)
            .map(ToPy)
    }

    /// Listen on web socket events
    pub fn listen_for_events(&mut self, event_filter: ToPy<FilterBox>) -> PyResult<EventIterator> {
        self.deref_mut()
            .listen_for_events(event_filter.into_inner())
            .map_err(to_py_err)
            .map(|iter| {
                let boxed = Box::new(iter);
                EventIterator::new(boxed)
            })
    }
}

// HACK: `EventIterator` was made private in iroha for some reason
#[pyclass]
pub struct EventIterator {
    inner: Box<dyn Iterator<Item = eyre::Result<Event>> + Send>,
}

impl EventIterator {
    fn new(inner: Box<dyn Iterator<Item = eyre::Result<Event>> + Send>) -> Self {
        Self { inner }
    }
}

#[pymethods]
impl EventIterator {
    fn __iter__(slf: PyRefMut<Self>) -> PyRefMut<Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<ToPy<Event>, &'static str> {
        #[allow(clippy::unwrap_used)]
        slf.inner
            .next()
            .map(Result::unwrap) // TODO:
            .map(ToPy)
            .map_or(IterNextOutput::Return("Ended"), IterNextOutput::Yield)
    }
}

#[pymethods]
impl SignedTransaction {
    #[staticmethod]
    /// Decode from hex representation of SCALE-encoded transaction
    fn decode(encoded: String) -> PyResult<Self> {
        let data = hex::decode(encoded).map_err(to_py_err)?;
        let tx = VersionedSignedTransaction::decode(&mut data.as_slice()).map_err(to_py_err)?;
        Ok(Self { tx })
    }

    /// Encode to hex representation of SCALE-encoded transaction
    fn encode(&self) -> String {
        let encoded = self.tx.encode();
        hex::encode(encoded)
    }

    /// Sign the transaction with provided key pair
    fn append_signature(&mut self, key_pair: KeyPair) -> PyResult<()> {
        let resigned = self
            .tx
            .as_v1()
            .clone()
            .sign(key_pair.deref().clone())
            .map_err(to_py_err)?;
        *self.tx.as_mut_v1() = resigned;
        Ok(())
    }
}

#[rustfmt::skip]
wrap_class!(
    KeyPair        { keys: iroha_crypto::KeyPair   }: Debug + Clone,
    Client         { cl:   client::Client          }: Debug + Clone,
    SignedTransaction { tx: iroha_data_model::transaction::VersionedSignedTransaction }: Debug + Clone,
);

/// A Python module implemented in Rust.
#[pymodule]
pub fn iroha2(_: Python, m: &PyModule) -> PyResult<()> {
    register_wrapped_classes(m)?;
    m.add_class::<types::Dict>()?;
    m.add_class::<types::List>()?;
    Ok(())
}
