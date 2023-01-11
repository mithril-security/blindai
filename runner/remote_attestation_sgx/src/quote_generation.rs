use aesm_client::AesmClient;
use dcap_ql::Quote3Error;
use log::debug;
use sgx_isa::Report;

// The RunnerContext object should define the quote generation steps
// by implementing as methods, the different procedures to take into
// account
#[derive(Debug, Clone)]
pub struct RunnerContext {
    pub aesm_client: AesmClient,
    pub target_info: Vec<u8>,
    pub ecdsa_key_id: Vec<u8>,
    pub report_slice: Vec<u8>,
    pub quote_slice: Vec<u8>,
}

const SGX_QL_ALG_ECDSA_P256: u32 = 2;

#[derive(Debug)]
pub enum RunnerError {
    IO(IOError),
    Aesm(aesm_client::Error),
    EnclaveNotTrusted,
    PseNotTrusted,
}
#[derive(Debug)]
pub enum IOError {
    Bincode(std::boxed::Box<bincode::ErrorKind>),
    StdIo(std::io::Error),
}

impl std::convert::From<aesm_client::Error> for RunnerError {
    fn from(e: aesm_client::Error) -> Self {
        Self::Aesm(e)
    }
}

impl std::convert::From<std::boxed::Box<bincode::ErrorKind>> for RunnerError {
    fn from(e: std::boxed::Box<bincode::ErrorKind>) -> Self {
        Self::IO(IOError::Bincode(e))
    }
}

impl std::convert::From<std::io::Error> for RunnerError {
    fn from(e: std::io::Error) -> Self {
        Self::IO(IOError::StdIo(e))
    }
}

pub type RunnerResult<T> = Result<T, RunnerError>;

pub fn get_algorithm_id(key_id: &[u8]) -> u32 {
    const ALGORITHM_OFFSET: usize = 154;

    let mut bytes: [u8; 4] = Default::default();
    bytes.copy_from_slice(&key_id[ALGORITHM_OFFSET..ALGORITHM_OFFSET + 4]);
    u32::from_le_bytes(bytes)
}

impl RunnerContext {
    // init of the RunnerContext
    // the target is initialized using the aesm deamon
    pub fn init() -> RunnerResult<Self> {
        let aesm_client = AesmClient::new();
        let key_ids = aesm_client.get_supported_att_key_ids().unwrap();
        debug!("key_ids : {:?}", key_ids);
        let ecdsa_key_id = key_ids
            .into_iter()
            .find(|id| SGX_QL_ALG_ECDSA_P256 == get_algorithm_id(id))
            .expect("[X] ECDSA attestation key not available");
        let quote_info = aesm_client.init_quote_ex(ecdsa_key_id.clone())?;
        let target_info = quote_info.target_info();
        let report_slice: Vec<u8> = Vec::new();
        let quote_slice: Vec<u8> = Vec::new();
        Ok(Self {
            aesm_client,
            target_info: target_info.to_vec(),
            ecdsa_key_id,
            report_slice,
            quote_slice,
        })
    }

    // Takes the report in bytes and returns the quote in Vec<u8>
    pub fn get_quote(&mut self) -> Result<Vec<u8>, Quote3Error> {
        debug!("report len : {:?}", &self.report_slice);
        let report = Report::try_copy_from(&self.report_slice[0..432]).unwrap();
        let ecdsa_key_id = self.ecdsa_key_id.clone();
        let quote = self
            .aesm_client
            .get_quote_ex(ecdsa_key_id, report.as_ref().to_owned(), None, vec![0; 16])
            .unwrap();
        self.quote_slice = quote.quote().to_vec();
        Ok(Vec::from(quote.quote()))
    }

    pub fn update_report(&mut self, report: Vec<u8>) {
        self.report_slice = report;
    }

    pub fn get_report(&mut self) -> Vec<u8> {
        self.report_slice.clone()
    }
}