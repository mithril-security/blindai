use std::{
    lazy::SyncOnceCell,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::client_communication::secured_exchange::ClientInfo;

use log::debug;
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedSender};

const AMPLITUDE_API_KEY: &str = "33888bd644f1dc39f72f2963c944c94c";

static TELEMETRY_CHANNEL: SyncOnceCell<UnboundedSender<TelemetryEvent>> = SyncOnceCell::new();

#[derive(Debug, Serialize)]
pub enum TelemetryEventProps {
    Started {},
    SendModel {
        model_name: Option<String>,
        model_size: usize,
        sign: bool,
        time_taken: f64,
    },
    RunModel {
        model_name: Option<String>,
        sign: bool,
        time_taken: f64,
    },
}

pub struct TelemetryEvent {
    props: TelemetryEventProps,
    time: SystemTime,
    client_info: Option<ClientInfo>,
}

impl TelemetryEventProps {
    fn event_type(&self) -> &'static str {
        match self {
            TelemetryEventProps::Started { .. } => "started",
            TelemetryEventProps::SendModel { .. } => "send_model",
            TelemetryEventProps::RunModel { .. } => "run_model",
        }
    }
}

pub fn add_event(event: TelemetryEventProps, client_info: Option<ClientInfo>) {
    if let Some(sender) = TELEMETRY_CHANNEL.get() {
        let _ = sender.send(TelemetryEvent {
            props: event,
            time: SystemTime::now(),
            client_info,
        });
    }
    // else, telemetry is disabled
}

#[derive(Debug, Serialize)]
struct RequestEvent<'a> {
    user_id: &'a str,
    event_type: &'a str,
    device_id: &'a str,
    time: u64,
    app_version: &'a str,
    user_properties: ReqestUserProperties<'a>,
    event_properties: Option<TelemetryEventProps>,
}

#[derive(Debug, Serialize, Default)]
struct ReqestUserProperties<'a> {
    sgx_mode: &'a str,
    uptime: u64,
    azure_dcsv3_patch_enabled: bool,
    client_uid: Option<String>,
    client_platform_name: Option<String>,
    client_platform_arch: Option<String>,
    client_platform_version: Option<String>,
    client_platform_release: Option<String>,
    client_user_agent: Option<String>,
    client_user_agent_version: Option<String>,
}

#[derive(Debug, Serialize)]
struct AmplitudeRequest<'a> {
    api_key: &'a str,
    events: &'a Vec<RequestEvent<'a>>,
}

pub fn setup(platform: String, uid: String) -> anyhow::Result<()> {
    let (sender, mut receiver) = mpsc::unbounded_channel::<TelemetryEvent>();

    TELEMETRY_CHANNEL.set(sender).unwrap();
    let sgx_mode = if cfg!(SGX_MODE = "SW") { "SW" } else { "HW" };

    let azure_dcsv3_patch_enabled = std::env::var("BLINDAI_AZURE_DCSV3_PATCH").is_ok();

    let first_start = SystemTime::now();
    tokio::task::spawn(async move {
        loop {
            let mut events = Vec::new();
            while let Ok(properties) = receiver.try_recv() {
                let event_type = properties.props.event_type();
                let mut user_properties = ReqestUserProperties {
                    uptime: properties
                        .time
                        .duration_since(first_start)
                        .unwrap()
                        .as_secs(),
                    sgx_mode,
                    azure_dcsv3_patch_enabled,
                    ..Default::default()
                };

                if let Some(client_info) = properties.client_info {
                    user_properties.client_uid = Some(client_info.uid);
                    user_properties.client_platform_name = Some(client_info.platform_name);
                    user_properties.client_platform_arch = Some(client_info.platform_arch);
                    user_properties.client_platform_version = Some(client_info.platform_version);
                    user_properties.client_platform_release = Some(client_info.platform_release);
                    user_properties.client_user_agent = Some(client_info.user_agent);
                    user_properties.client_user_agent_version = Some(client_info.user_agent_version);
                }

                let event = RequestEvent {
                    user_id: &uid,
                    event_type,
                    device_id: &platform,
                    time: properties
                        .time
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    app_version: env!("CARGO_PKG_VERSION"),
                    user_properties,
                    event_properties: Some(properties.props),
                };

                events.push(event);
            }

            let request = AmplitudeRequest {
                api_key: AMPLITUDE_API_KEY,
                events: &events,
            };

            if !events.is_empty() {
                let response = reqwest::Client::new()
                    .post("https://api2.amplitude.com/2/httpapi")
                    .timeout(Duration::from_secs(60))
                    .json(&request)
                    .send()
                    .await;
                if let Err(e) = response {
                    debug!("Cannot contact telemetry server: {}", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    Ok(())
}
