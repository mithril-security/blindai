use aesm_client::AesmClient;
use anyhow::{anyhow, ensure, Result};
use log::debug;
use sgx_isa::{Report, Targetinfo};

const SGX_QL_ALG_ECDSA_P256: u32 = 2;
pub struct QuoteProvider {
    aesm_client: AesmClient,
    target_info: Targetinfo,
    ecdsa_key_id: Vec<u8>,
}

fn get_algorithm_id(key_id: &[u8]) -> u32 {
    const ALGORITHM_OFFSET: usize = 154;
    let mut bytes: [u8; 4] = Default::default();
    bytes.copy_from_slice(&key_id[ALGORITHM_OFFSET..ALGORITHM_OFFSET + 4]);
    u32::from_le_bytes(bytes)
}

impl QuoteProvider {
    pub fn init() -> Result<Self> {
        let aesm_client = AesmClient::new();
        let key_ids = aesm_client.get_supported_att_key_ids().unwrap();
        debug!("key_ids : {:?}", key_ids);
        let ecdsa_key_ids: Vec<_> = key_ids
            .into_iter()
            .filter(|id| SGX_QL_ALG_ECDSA_P256 == get_algorithm_id(id))
            .collect();
        ensure!(
            ecdsa_key_ids.len() == 1,
            "Expected exactly one ECDSA attestation key, got {} key(s) instead",
            ecdsa_key_ids.len()
        );
        let ecdsa_key_id = ecdsa_key_ids[0].to_vec();
        let quote_info = aesm_client
            .init_quote_ex(ecdsa_key_id.clone())
            .map_err(|e| anyhow::anyhow!(e))?;
        let target_info = Targetinfo::try_copy_from(quote_info.target_info())
            .ok_or(anyhow!("Invalid target info"))?;
        Ok(QuoteProvider {
            aesm_client,
            target_info,
            ecdsa_key_id,
        })
    }
    pub fn get_target_info(&self) -> Targetinfo {
        self.target_info.clone()
    }
    pub fn get_quote(&self, report: Report) -> Result<Vec<u8>> {
        let ecdsa_key_id = self.ecdsa_key_id.clone();

        // Security : The nonce is set to 0
        //
        // While in some instances reusing nonces can lead to security vulnerability this is not the case here.
        //
        // Here is an extract from Intel code [1]
        // > The caller can request a REPORT from the QE using a supplied nonce. This will allow
        // > the enclave requesting the quote to verify the QE used to generate the quote. This
        // > makes it more difficult for something to spoof a QE and allows the app enclave to
        // > catch it earlier. But since the authenticity of the QE lies in the knowledge of the
        // > Quote signing key, such spoofing will ultimately be detected by the quote verifier.
        // > QE REPORT.ReportData = SHA256(*p_{nonce}||*p_{quote})||0x00)
        //
        // Since setting a nonce would add no measurable security benefit in our threat model,
        // we chose not to do so, because it would only add complexity.
        //
        // [1] <https://github.com/intel/linux-sgx/blob/26c458905b72e66db7ac1feae04b43461ce1b76f/common/inc/sgx_uae_quote_ex.h#L158>

        let quote_result = self
            .aesm_client
            .get_quote_ex(ecdsa_key_id, report.as_ref().to_owned(), None, vec![0; 16])
            .unwrap();

        Ok(quote_result.quote().to_vec())
    }
}
