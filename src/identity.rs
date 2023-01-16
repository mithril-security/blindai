// Copyright 2022 Mithril Security. All rights reserved.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use anyhow::{anyhow, Result};
use der_parser::oid;
use rand::{rngs::OsRng, RngCore};
use rcgen::{Certificate, CertificateParams, CustomExtension, SanType};
use ring_compat::signature::ed25519::SigningKey;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::borrow::Borrow;
use x509_parser::prelude::{FromDer, X509Certificate};

impl<T> From<T> for TlsIdentity
where
    T: Borrow<Certificate>,
{
    fn from(cert: T) -> Self {
        let cert = cert.borrow();
        // Get DER cert and key from the certificate
        let cert_der = cert.serialize_pem().unwrap().as_bytes().to_vec();
        let private_key_der = cert.serialize_private_key_pem().as_bytes().to_vec();

        // Create Identity from the PEM key-pair
        TlsIdentity {
            cert_der,
            private_key_der,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TlsIdentity {
    // TLS certificate and private key
    pub cert_der: Vec<u8>,        // DER encoded X509 certificate
    pub private_key_der: Vec<u8>, // DER encoded private key
}

#[derive(Serialize, Deserialize)]

pub(crate) struct RsaKeyPair {
    pub private_key_der: Vec<u8>,
    pub public_key_der: Vec<u8>,
}

pub(crate) struct MyIdentity {
    pub tls_identity: TlsIdentity,
    // Key pair for secure storage
    #[allow(unused)]
    signing_key_seed: Vec<u8>,
    pub signing_key: SigningKey,
}

impl MyIdentity {
    pub fn from_cert(certificate: Certificate, signing_key_seed: Vec<u8>) -> Self {
        let tls_identity = TlsIdentity::from(&certificate);

        MyIdentity {
            tls_identity,
            signing_key: SigningKey::from_seed(&signing_key_seed).unwrap(),
            signing_key_seed,
        }
    }

    #[allow(dead_code)]
    pub fn uid(&self) -> Result<String> {
        /* extract the public key from the certificate */
        let (_, cert) = X509Certificate::from_der(&self.tls_identity.cert_der)?;
        Ok(hex::encode(cert.public_key().raw))
    }
}

pub(crate) fn create_certificate() -> Result<(Certificate, Vec<u8>)> {
    // Generate a self signed certificate

    let subject_alt_names: &[_] = &["blindai-srv".to_string()];

    let subject_alt_names = subject_alt_names
        .iter()
        .map(|s| SanType::DnsName(s.to_string()))
        .collect::<Vec<_>>();

    let payload_signing_key_oid: Vec<_> = oid!(1.3.6 .1 .3 .2)
        .iter()
        .ok_or_else(|| anyhow!("At least one arc of the OID does not fit into `u64`"))?
        .collect();

    let mut params = CertificateParams::default();
    params.subject_alt_names = subject_alt_names;

    /* todo! : force the right params */

    let mut ed25519_seed = [0u8; 32];
    OsRng.fill_bytes(&mut ed25519_seed);

    let signing_key = SigningKey::from_seed(&ed25519_seed).unwrap();
    let verify_key = signing_key.verify_key();

    let ext2 =
        CustomExtension::from_oid_content(&payload_signing_key_oid, verify_key.as_ref().to_vec());

    params.custom_extensions = vec![ext2];

    Ok((
        Certificate::from_params(params)?,
        // rsa_key_pair,
        ed25519_seed.to_vec(),
    ))
}
