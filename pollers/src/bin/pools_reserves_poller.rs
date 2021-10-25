use pollers::pools_reserves_poller::PoolsExtractor;

#[tokio::main]
async fn main() {
    PoolsExtractor::new().get_pool_data().await;
}
