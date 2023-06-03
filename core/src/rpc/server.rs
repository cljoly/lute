use std::sync::Arc;

use crate::{
  events::event_publisher::EventPublisher,
  files::file_service::FileService,
  proto::{FileServiceServer, HealthCheckReply, Lute, LuteServer},
  settings::Settings,
};
use tonic::{transport::Server, Request, Response, Status};

pub struct LuteService {
  redis_connection_pool: Arc<r2d2::Pool<redis::Client>>,
}

#[tonic::async_trait]
impl Lute for LuteService {
  async fn health_check(&self, request: Request<()>) -> Result<Response<HealthCheckReply>, Status> {
    println!("Got a request: {:?}", request);

    let reply = HealthCheckReply { ok: true };

    Ok(Response::new(reply))
  }
}

pub struct RpcServer {
  settings: Settings,
  redis_connection_pool: Arc<r2d2::Pool<redis::Client>>,
  event_publisher: Arc<EventPublisher>,
}

impl RpcServer {
  pub fn new(
    settings: Settings,
    redis_connection_pool: Arc<r2d2::Pool<redis::Client>>,
    event_publisher: Arc<EventPublisher>,
  ) -> Self {
    Self {
      settings,
      redis_connection_pool,
      event_publisher,
    }
  }

  pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
    let lute_service = LuteService {
      redis_connection_pool: self.redis_connection_pool.clone(),
    };
    let file_service = FileService::new(
      self.settings.file.clone(),
      self.redis_connection_pool.clone(),
      self.event_publisher.clone(),
    );

    let addr = "127.0.0.1:22000".parse().unwrap();

    println!("Lute listening on {}", addr);

    Server::builder()
      .accept_http1(true)
      .add_service(tonic_web::enable(LuteServer::new(lute_service)))
      .add_service(tonic_web::enable(FileServiceServer::new(file_service)))
      .serve(addr)
      .await?;

    Ok(())
  }
}
