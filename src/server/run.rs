use actix_web::{
    dev::Server,
    http::StatusCode,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpResponseBuilder, HttpServer,
};
use prometheus::Encoder;
use tokio::sync::watch;
use tracing::info;

use crate::{
    rpc::RpcClient,
    types::{MonError, RuntimeConfig},
    Metrics,
};

pub fn run(
    listen_addr: String,
    port: u16,
    rpc_url: String,
    config: RuntimeConfig,
    receiver: watch::Receiver<crate::types::Data>,
) -> Result<Server, MonError> {
    let metrics = Metrics::new();
    metrics.register()?;

    let rpc_client = RpcClient::new(rpc_url)?;

    let server = HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(health_check))
            .route("/metrics", web::get().to(get_metrics))
            .app_data(web::Data::new(rpc_client.clone()))
            .app_data(web::Data::new(metrics.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(receiver.clone()))
    })
    .bind((listen_addr, port))?
    .run();

    Ok(server)
}

async fn get_metrics(
    req: HttpRequest,
    rpc_client: Data<RpcClient>,
    metrics: Data<Metrics>,
    config: Data<RuntimeConfig>,
    receiver: Data<watch::Receiver<crate::types::Data>>,
) -> Result<HttpResponse, MonError> {
    info!("Request to: {}", req.head().uri);

    // Update from rpc
    metrics.update_for_rpc(&rpc_client.into_inner()).await?;

    // Update for the data from monitoring
    let data = receiver.borrow().clone();
    metrics.update_for_monitoring_data(&data).await?;

    let (encoder, mut buffer) = metrics.get_encoder_and_buffer()?;
    let block_window_metric = format!("\n# HELP ftso_search_window The ftso block search window\n# TYPE ftso_search_window gauge\nftso_search_window '{}'\n", config.block_window).as_bytes().to_vec();

    buffer.extend(&block_window_metric);

    Ok(HttpResponseBuilder::new(StatusCode::OK)
        .insert_header(("Content-Type", encoder.format_type()))
        .body(buffer))
}

async fn health_check(req: HttpRequest) -> HttpResponse {
    info!("Request to: {}", req.head().uri);
    HttpResponse::Ok().finish()
}
