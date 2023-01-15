#![feature(type_alias_impl_trait)]

mod quote_generation;
mod quote_verification_collateral;

use anyhow::Result;
use quote_generation::QuoteProvider;
use quote_verification_collateral::get_quote_verification_collateral;

use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router, extract::State};
use serde::Deserialize;
use serde_json::json;
use sgx_isa::Report;
use std::{net::SocketAddr, sync::Arc};

#[tokio::main(flavor = "current_thread")]
pub async fn start_remote_attestation() {
    let app = Router::new()
        .route("/get_target_info", post(get_target_info))
        .route("/get_quote", post(get_quote))
        .route("/get_collateral", post(get_collateral))
        .with_state(Arc::new(QuoteProvider::init().unwrap()));

    let addr = SocketAddr::from(([127, 0, 0, 1], 11000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct GetQuoteRequest {
    enclave_report: Report,
}

struct WebError(anyhow::Error);

impl IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        // its often easiest to implement `IntoResponse` by calling other implementations
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong :{}", self.0),
        )
            .into_response()
    }
}

type WebResult = Result<impl IntoResponse, WebError>;
impl<E> From<E> for WebError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn get_target_info(State(quote_provider): State<Arc<QuoteProvider>>) -> WebResult {
    Ok(Json(json! { quote_provider.get_target_info() }))
}

async fn get_quote(State(quote_provider): State<Arc<QuoteProvider>>, Json(GetQuoteRequest { enclave_report }): Json<GetQuoteRequest>) -> WebResult {
    Ok(Json(json! { quote_provider.get_quote(enclave_report)? }))
}

#[derive(Deserialize)]
struct GetCollateralRequest {
    quote: Vec<u8>,
}

async fn get_collateral(
    Json(GetCollateralRequest { quote }): Json<GetCollateralRequest>,
) -> WebResult {
    Ok(Json(json! { get_quote_verification_collateral(&quote)? }))
}
