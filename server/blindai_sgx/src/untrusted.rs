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

use crate::dcap_quote_provider::DcapQuoteProvider;
use blindai_common::untrusted_local_app_client::UntrustedLocalAppClient;
use tonic::{Request, Response, Status};
pub use untrusted::attestation_server::*;
use untrusted::*;

pub mod untrusted {
    tonic::include_proto!("untrusted");
}

// #[derive(Default)]
pub struct MyAttestation {
    // pub untrusted: UntrustedLocalAppClient<Channel>,
    pub(crate) quote_provider: &'static DcapQuoteProvider,
}

#[tonic::async_trait]
impl Attestation for MyAttestation {
    async fn get_certificate(
        &self,
        _request: Request<GetCertificateRequest>,
    ) -> Result<Response<GetCertificateReply>, Status> {
        Ok(Response::new(GetCertificateReply {
            enclave_tls_certificate: self.quote_provider.enclave_held_data.clone(),
        }))
    }
    async fn get_token(
        &self,
        request: Request<GetTokenRequest>,
    ) -> Result<Response<GetTokenReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = untrusted::GetTokenReply {
            token: "".to_owned(), // token: self.token.lock().unwrap().clone()
        };
        Ok(Response::new(reply))
    }

    async fn get_sgx_quote_with_collateral(
        &self,
        _request: Request<GetSgxQuoteWithCollateralRequest>,
    ) -> Result<Response<GetSgxQuoteWithCollateralReply>, Status> {
        if cfg!(SGX_MODE = "SW") {
            return Err(Status::unimplemented(
                "Attestation is not available. Running in Simulation Mode",
            ));
        }

        let quote = self.quote_provider.get_quote().unwrap();

        let mut untrusted = UntrustedLocalAppClient::connect("http://127.0.0.1:50053")
            .await
            .unwrap();
        let collateral = untrusted
            .get_collateral_from_quote(quote.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?
            .into_inner();

        let reply = GetSgxQuoteWithCollateralReply {
            collateral: Some(SgxCollateral {
                version: collateral.version, // version = 1.  PCK Cert chain is in the Quote.
                pck_crl_issuer_chain: collateral.pck_crl_issuer_chain.into(),
                root_ca_crl: collateral.root_ca_crl.into(), // Root CA CRL
                pck_crl: collateral.pck_crl.into(),         // PCK Cert CRL
                tcb_info_issuer_chain: collateral.tcb_info_issuer_chain.into(),
                tcb_info: collateral.tcb_info.into(), // TCB Info structure
                qe_identity_issuer_chain: collateral.qe_identity_issuer_chain.into(),
                qe_identity: collateral.qe_identity.into(), // QE Identity Structure
                pck_certificate: collateral.pck_certificate, //PEM encoded PCK certificate
                pck_signing_chain: collateral.pck_signing_chain, // PEM encoded PCK signing chain such that (pck_certificate || pck_signing_chain) == pck_cert_chain
            }),
            quote: quote,
            enclave_held_data: self.quote_provider.enclave_held_data.clone(),
        };
        Ok(Response::new(reply))
    }
}
