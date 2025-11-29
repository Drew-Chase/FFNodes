#[actix_web::main]
async fn main()->anyhow::Result<()>{
	ffnodes_server_lib::run().await
}
