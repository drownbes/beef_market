use super::{Product, Scraper};
use crate::scraper::PriceEur;
use async_trait::async_trait;
use reqwest::Url;
use rust_decimal::prelude::*;
use scraper::{selectable::Selectable, Html, Selector};
use tracing::{info, instrument};

struct Selver {
    url: Url,
}

#[async_trait]
impl Scraper for Selver {
    #[instrument(skip(self))]
    async fn run(&self) -> anyhow::Result<Vec<Product>> {
        let html = reqwest::get(self.url.as_ref())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let document = Html::parse_document(&html);
        let product_listing_sel = Selector::parse(".ProductListing").unwrap();
        let product_card_sel = Selector::parse(".ProductCard").unwrap();
        let product = document.select(&product_listing_sel).next().unwrap();
        let product_cards = product.select(&product_card_sel);
        let price_sel = Selector::parse(".ProductPrice__unit-price").unwrap();
        let product_title_sel = Selector::parse(".ProductCard__title").unwrap();

        let products: Vec<Product> = product_cards
            .filter_map(|card| {
                let product_name: &str = card
                    .select(&product_title_sel)
                    .next()?
                    .text()
                    .nth(0)?
                    .trim_matches('\n')
                    .trim();

                dbg!(&product_name);

                let price = card
                    .select(&price_sel)
                    .next()?
                    .text()
                    .nth(0)?
                    .strip_suffix("€/kg")?
                    .trim()
                    .split('\n')
                    .nth(0)?
                    .trim()
                    .replace(",", ".");
                dbg!(&price);

                let price: Decimal = Decimal::from_str(&price).ok()?;

                Some(Product {
                    name: product_name.into(),
                    price: PriceEur(price),
                })
            })
            .collect();

        info!("Scraped {} beef products", products.len());
        Ok(products)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_selver_parsing() {
        let base_url = Url::parse("https://www.selver.ee/").expect("Failed to parse base_url");

        let path_to_beef =
            "/liha-ja-kalatooted/veise-lamba-ja-ulukiliha?product_segment=4725".to_string();

        let url = base_url.join(&path_to_beef).unwrap();

        let b = Selver { url };

        let products = b.run().await.unwrap();
        dbg!(products);
    }
}
