use crate::gton_price_poller::PriceExtractor;

#[tokio::main]
async fn main() {
    PriceExtractor::new().run().await;
}
