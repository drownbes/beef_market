use super::{Product, Scraper};
use crate::scraper::PriceEur;
use async_trait::async_trait;
use reqwest::Url;
use rust_decimal::prelude::*;
use scraper::{selectable::Selectable, Html, Selector};
use tracing::{info, instrument};

struct Rimi {
    url: Url,
}

#[async_trait]
impl Scraper for Rimi {
    #[instrument(skip(self))]
    async fn run(&self) -> anyhow::Result<Vec<Product>> {
        let html = reqwest::get(self.url.as_ref())
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let document = Html::parse_document(&html);
        let product_listing_sel = Selector::parse(".product-grid").unwrap();
        let product_card_sel = Selector::parse(".product-grid__item").unwrap();
        let product = document.select(&product_listing_sel).next().unwrap();
        let product_cards = product.select(&product_card_sel);
        let price_sel = Selector::parse(".card__price-per").unwrap();
        let product_title_sel = Selector::parse(".card__name").unwrap();

        let products: Vec<Product> = product_cards
            .filter_map(|card| {
                let product_name = card
                    .select(&product_title_sel)
                    .next()?
                    .text()
                    .nth(0)?
                    .trim_matches('\n')
                    .trim();
                let price: String = card
                    .select(&price_sel)
                    .next()?
                    .text()
                    .nth(0)?
                    .trim()
                    .split('\n')
                    .nth(0)?
                    .trim()
                    .replace(",", ".");
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
    use crate::logger::init_tracing;

    use super::*;

    #[tokio::test]
    async fn test_rimi_parsing() {
        init_tracing();
        let base_url = Url::parse("https://rimi.ee/").expect("Failed to parse base_url");

        let path_to_beef =
            "/epood/ee/tooted/liha--ja-kalatooted/veise--lamba--ja-ulukiliha/c/SH-8-21/"
                .to_string();
        let url = base_url.join(&path_to_beef).unwrap();

        let b = Rimi { url };

        let products = b.run().await.unwrap();
        dbg!(products);
    }
}
