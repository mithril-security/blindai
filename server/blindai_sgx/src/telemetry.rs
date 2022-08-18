use std::{
    lazy::SyncOnceCell,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::client_communication::secured_exchange::ClientInfo;

use log::debug;
use serde::Serialize;
use tokio::sync::mpsc::{self, UnboundedSender};

static TELEMETRY_CHANNEL: SyncOnceCell<UnboundedSender<TelemetryEvent>> = SyncOnceCell::new();

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone)]
pub struct TelemetryEvent {
    event_type: &'static str,
    is_client_event: bool,
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
    fn user_event_type(&self) -> Option<&'static str> {
        match self {
            TelemetryEventProps::Started { .. } => None,
            TelemetryEventProps::SendModel { .. } => Some("client_send_model"),
            TelemetryEventProps::RunModel { .. } => Some("client_run_model"),
        }
    }
}

pub fn add_event(event: TelemetryEventProps, client_info: Option<ClientInfo>) {
    if let Some(sender) = TELEMETRY_CHANNEL.get() {
        if client_info.is_some() && event.user_event_type().is_some() {
            // send the event as a user event too
            let _ = sender.send(TelemetryEvent {
                is_client_event: true,
                event_type: event.user_event_type().unwrap(),
                props: event.clone(),
                time: SystemTime::now(),
                client_info: client_info.clone(),
            });
        }
        let _ = sender.send(TelemetryEvent {
            is_client_event: false,
            event_type: event.event_type(),
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
            {
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
                        user_properties.client_platform_name =
                            Some(client_info.platform_name.as_ref());
                        user_properties.client_platform_arch =
                            Some(client_info.platform_arch.as_ref());
                        user_properties.client_platform_version =
                            Some(client_info.platform_version.as_ref());
                        user_properties.client_platform_release =
                            Some(client_info.platform_release.as_ref());
                        user_properties.client_user_agent = Some(client_info.user_agent.as_ref());
                        user_properties.client_user_agent_version =
                            Some(client_info.user_agent_version.as_ref());
                    }

                    let event_type = properties.event_type;
                    let (user_id, device_id, app_version) = {
                        let client_info = properties.client_info.as_ref();
                        if properties.is_client_event && client_info.is_some() {
                            // this is a client event
                            let ua = client_info.unwrap();
                            (
                                ua.uid.as_ref(),
                                ua.user_agent.as_ref(),
                                ua.user_agent_version.as_ref(),
                            )
                        } else {
                            // this is a server event
                            (uid.as_ref(), platform.as_ref(), env!("CARGO_PKG_VERSION"))
                        }
                    };

                    let event = RequestEvent {
                        user_id,
                        event_type,
                        device_id,
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

                //We send using the server, the differents event in the db
                if !events.is_empty() {
                    let response = reqwest::Client::new()
                        .post("https://telemetry.mithrilsecurity.io")
                        .timeout(Duration::from_secs(60))
                        .json(&events)
                        .send()
                        .await;
                    if let Err(e) = response {
                        debug!("Cannot contact telemetry server: {}", e);
                    }
                };
            }

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });

    Ok(())
}