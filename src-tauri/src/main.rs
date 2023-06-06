use actix_web::{get, App, HttpServer, Responder};
use kube::api::Api;
use kube::config;
use kube::error::Result;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct Pod {
    name: String,
    
}

#[get("/pods")]
async fn list_pods() -> impl Responder {
    let kubeconfig = config::load_kube_config().await.unwrap();
    let client = kube::Client::new(kubeconfig);
    let pods: Api<kube::api::Pod> = Api::namespaced(client, "default");
    let pod_list = pods.list(&Default::default()).await.unwrap();
    let pod_names = pod_list.items.into_iter()
        .map(|pod| Pod { name: pod.metadata.name.unwrap() })
        .collect::<Vec<Pod>>();
    actix_web::HttpResponse::Ok().json(pod_names)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(list_pods))
        .bind("1420")?
        .run()
        .await
}
