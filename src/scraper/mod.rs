mod barbora;
mod rimi;
mod selver;
use async_trait::async_trait;
use rust_decimal::Decimal;

#[derive(Debug)]
pub struct PriceEur(Decimal);

#[derive(Debug)]
pub struct Product {
    name: String,
    price: PriceEur,
}

#[async_trait]
pub trait Scraper {
    async fn run(&self) -> anyhow::Result<Vec<Product>>;
}
