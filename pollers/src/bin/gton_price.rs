use pollers::gton_price_poller::PricePoller;

#[tokio::main]
async fn main() {
    PricePoller::new().run().await;
}
