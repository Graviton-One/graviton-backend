use pollers::farm_updater::FarmExtractor;

#[tokio::main]
async fn main() {
    FarmExtractor::new().update_stakes().await;
}
