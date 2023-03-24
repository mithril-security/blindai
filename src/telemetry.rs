use crate::client_communication::ClientInfo;
use crate::ureq_dns_resolver::InternalAgent;
use crate::TELEMETRY_CHANNEL;
use serde::Serialize;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const TELEMETRY_IP: &str = "163.172.188.78";
const TELEMETRY_URL: &str = "https://telemetry.mithrilsecurity.io/blindai";

#[derive(Debug, Clone, Serialize)]
pub enum TelemetryEventProps {
    Started {},
    SendModel {
        model_name: Option<String>,
        model_size: usize,
        // sign: bool, Used when we will support signed models
        time_taken: f64,
    },
    RunModel {
        model_hash: Option<String>,
        // sign: bool, Used when we will support signed models
        time_taken: f64,
    },
}

#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    event_type: &'static str,
    props: TelemetryEventProps,
    time: SystemTime,
    client_info: Option<ClientInfo>,
    cloud_user_id: Option<usize>,
}

impl TelemetryEventProps {
    fn event_type(&self) -> &'static str {
        match self {
            TelemetryEventProps::Started {} => "started",
            TelemetryEventProps::SendModel { .. } => "send_model",
            TelemetryEventProps::RunModel { .. } => "run_model",
        }
    }
}

pub(crate) fn add_event(
    event: TelemetryEventProps,
    client_info: Option<ClientInfo>,
    cloud_user_id: Option<usize>,
) {
    let channel = TELEMETRY_CHANNEL.get_sender();
    if let Some(sender) = channel {
        let sender = sender.lock().unwrap();
        let _ = sender.send(TelemetryEvent {
            event_type: event.event_type(),
            props: event,
            time: SystemTime::now(),
            client_info,
            cloud_user_id,
        });
    }
}

#[derive(Debug, Serialize)]
struct RequestEvent<'a> {
    user_id: &'a str,
    cloud_user_id: Option<usize>,
    custom_agent_id: Option<&'a str>,
    event_type: &'a str,
    device_id: &'a str,
    time: u64,
    app_version: &'a str,
    user_properties: ReqestUserProperties<'a>,
    event_properties: Option<&'a TelemetryEventProps>,
}

#[derive(Debug, Serialize, Default)]
struct ReqestUserProperties<'a> {
    sgx_mode: &'a str,
    uptime: u64,
    azure_dcsv3_patch_enabled: bool,
    client_uid: Option<&'a str>,
    client_platform_name: Option<&'a str>,
    client_platform_arch: Option<&'a str>,
    client_platform_version: Option<&'a str>,
    client_platform_release: Option<&'a str>,
    client_user_agent: Option<&'a str>,
    client_user_agent_version: Option<&'a str>,
}

pub fn setup() -> anyhow::Result<bool> {
    let sgx_mode = if cfg!(target_env = "sgx") { "HW" } else { "SW" };
    let azure_dcsv3_patch_enabled = std::env::var("BLINDAI_AZURE_DCSV3_PATCH").is_ok();

    let first_start = SystemTime::now();

    thread::spawn(move || loop {
        let receiver = TELEMETRY_CHANNEL.get_receiver().unwrap();
        let custom_agent_id = TELEMETRY_CHANNEL.get_custom_agent_id();
        let uid = TELEMETRY_CHANNEL.get_uid().unwrap_or(String::default());
        let platform = TELEMETRY_CHANNEL
            .get_platform()
            .unwrap_or(String::default());
        let receiver = receiver.lock().unwrap();

        let mut received_events = Vec::new();
        let mut events = Vec::new();
        while let Ok(properties) = receiver.try_recv() {
            received_events.push(properties);
        }

        for properties in &received_events {
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

            if let Some(ref client_info) = properties.client_info {
                user_properties.client_uid = Some(client_info.uid.as_ref());
                user_properties.client_platform_name = Some(client_info.platform_name.as_ref());
                user_properties.client_platform_arch = Some(client_info.platform_arch.as_ref());
                user_properties.client_platform_version =
                    Some(client_info.platform_version.as_ref());
                user_properties.client_platform_release =
                    Some(client_info.platform_release.as_ref());
                user_properties.client_user_agent = Some(client_info.user_agent.as_ref());
                user_properties.client_user_agent_version =
                    Some(client_info.user_agent_version.as_ref());
            }

            let event_type = properties.event_type;
            let (user_id, device_id, app_version) =
                (uid.as_ref(), platform.as_ref(), env!("CARGO_PKG_VERSION"));

            let event = RequestEvent {
                user_id,
                event_type,
                device_id,
                cloud_user_id: properties.cloud_user_id,
                custom_agent_id,
                time: properties
                    .time
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                app_version,
                user_properties,
                event_properties: Some(&properties.props),
            };

            events.push(event);
        }

        if !events.is_empty() {
            let agent = InternalAgent::new(TELEMETRY_IP, "443", &["telemetry.mithrilsecurity.io"]);
            let response = agent.post(TELEMETRY_URL).send_json(&events);

            if let Err(e) = response {
                log::debug!("Cannot contact telemetry server: {}", e);
            }
        };
        thread::sleep(Duration::from_secs(5));
    });
    Ok(false)
}

pub struct Telemetry {
    disabled: bool,
    sender: Option<Arc<Mutex<Sender<TelemetryEvent>>>>,
    receiver: Option<Arc<Mutex<Receiver<TelemetryEvent>>>>,
    custom_agent_id: Option<String>,
    platform: Option<String>,
    uid: Option<String>,
}

impl Telemetry {
    pub fn new() -> anyhow::Result<Self> {
        let telemetry_disabled = std::env::var("BLINDAI_DISABLE_TELEMETRY").is_ok()
            || std::env::var("DO_NOT_TRACK").is_ok();

        let init = if telemetry_disabled {
            Self {
                disabled: true,
                sender: None,
                receiver: None,
                custom_agent_id: None,
                platform: None,
                uid: None,
            }
        } else {
            let (sender, receiver) = mpsc::channel::<TelemetryEvent>();

            let get_arg = |name: &str| -> String {
                let args = std::env::args();
                for arg in args {
                    if arg.starts_with(name) {
                        return arg.split('=').last().unwrap().to_string();
                    }
                }
                String::default()
            };

            let custom_agent_id = get_arg("--custom-agent-id");
            let platform = get_arg("--platform");
            let uid = get_arg("--uid");

            setup()?;

            Self {
                disabled: false,
                sender: Some(Arc::new(Mutex::new(sender))),
                receiver: Some(Arc::new(Mutex::new(receiver))),
                custom_agent_id: Some(custom_agent_id),
                platform: Some(platform),
                uid: Some(uid),
            }
        };

        Ok(init)
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    pub fn get_sender(&self) -> Option<Arc<Mutex<Sender<TelemetryEvent>>>> {
        self.sender.clone()
    }

    pub fn get_receiver(&self) -> Option<Arc<Mutex<Receiver<TelemetryEvent>>>> {
        self.receiver.clone()
    }

    pub fn get_custom_agent_id(&self) -> Option<&str> {
        self.custom_agent_id.as_deref()
    }

    pub fn get_platform(&self) -> Option<String> {
        self.platform.clone()
    }

    pub fn get_uid(&self) -> Option<String> {
        self.uid.clone()
    }
}
