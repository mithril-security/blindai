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
use pkix::pem::{PEM_CERTIFICATE, PEM_PRIVATE_KEY};
use rand::rngs::OsRng;
use rcgen::{Certificate, CertificateParams, CustomExtension, SanType};
use rsa::{
    pkcs1::{ToRsaPrivateKey, ToRsaPublicKey},
    RsaPrivateKey, RsaPublicKey,
};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use tonic::transport::Identity;
use x509_parser::{prelude::X509Certificate, traits::FromDer};

impl<T> From<T> for TlsIdentity
where
    T: Borrow<Certificate>,
{
    fn from(cert: T) -> Self {
        let cert = cert.borrow();
        // Get DER cert and key from the certificate
        let cert_der = cert.serialize_der().unwrap();
        let private_key_der = cert.serialize_private_key_der();

        // Create Identity from the PEM key-pair
        TlsIdentity {
            cert_der: cert_der,
            private_key_der: private_key_der,
        }
    }
}

impl From<&TlsIdentity> for Identity {
    fn from(identity: &TlsIdentity) -> Self {
        // Convert from DER format to PEM
        let cert_pem = pkix::pem::der_to_pem(&identity.cert_der, PEM_CERTIFICATE);
        let private_key_pem = pkix::pem::der_to_pem(&identity.private_key_der, PEM_PRIVATE_KEY);
        Identity::from_pem(cert_pem, private_key_pem)
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct TlsIdentity {
    // TLS certificate and private key
    pub cert_der: Vec<u8>,    // DER encoded X509 certificate
    private_key_der: Vec<u8>, // DER encoded private key
}

#[derive(Serialize, Deserialize)]

pub(crate) struct RsaKeyPair {
    pub private_key_der: Vec<u8>,
    pub public_key_der: Vec<u8>,
}
#[derive(Serialize, Deserialize)]
pub(crate) struct MyIdentity {
    pub tls_identity: TlsIdentity,
    // Key pair for secure storage
    pub storage_identity: RsaKeyPair,
}

impl MyIdentity {
    pub fn from_cert(certificate: Certificate, storage_identity: RsaKeyPair) -> Self {
        let tls_identity = TlsIdentity::from(&certificate);

        MyIdentity {
            tls_identity,
            storage_identity,
        }
    }

    #[allow(dead_code)]
    pub fn uid(&self) -> Result<String> {
        /* extract the public key from the certificate */
        let (_, cert) = X509Certificate::from_der(&self.tls_identity.cert_der)?;
        Ok(hex::encode(cert.public_key().raw))
    }
}

pub(crate) fn create_certificate() -> Result<(Certificate, RsaKeyPair)> {
    // Generate a self signed certificate

    let subject_alt_names: &[_] = &["blindai-srv".to_string()];

    let subject_alt_names = Vec::from(subject_alt_names)
        .into_iter()
        .map(|s| SanType::DnsName(s))
        .collect::<Vec<_>>();

    let mut params = CertificateParams::default();
    params.subject_alt_names = subject_alt_names;

    /* OIDs under the Internet Experimental OID arc (1.3.6.1.3.x) may be used for experimental purpose */
    let rsa_file_encryption_key_oid: Vec<_> = oid!(1.3.6 .1 .3 .1)
        .iter()
        .ok_or(anyhow!(
            "At least one arc of the OID does not fit into `u64`"
        ))?
        .collect();

    /* create random RSA key pair */
    let mut rng = OsRng;
    let bits = 2048;
    let rsa_private_key = RsaPrivateKey::new(&mut rng, bits)?;
    let rsa_public_key = RsaPublicKey::from(&rsa_private_key);

    /* convert the RSA private key to pkcs1 DER byte array */
    let rsa_private_key_doc = rsa_private_key.to_pkcs1_der()?;
    let rsa_private_key_bytes = rsa_private_key_doc.as_der().to_vec();

    /* convert the RSA public key to pkcs1 DER byte array */
    let rsa_public_key_doc = rsa_public_key.to_pkcs1_der()?;
    let rsa_public_key_bytes = rsa_public_key_doc.as_der().to_vec();

    let rsa_key_pair = RsaKeyPair {
        public_key_der: rsa_public_key_bytes.clone(),
        private_key_der: rsa_private_key_bytes,
    };

    /* add the RSA public key as bytes to the certificate */
    let signing_ext = CustomExtension::from_oid_content(
        &rsa_file_encryption_key_oid,
        rsa_public_key_bytes.clone(),
    );

    params.custom_extensions = vec![signing_ext];
    /* todo! : force the right params */

    Ok((Certificate::from_params(params)?, rsa_key_pair))
}
