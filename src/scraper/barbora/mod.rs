use crate::scraper::PriceEur;
use async_trait::async_trait;
use reqwest::Url;
use tracing::{info, instrument};

use super::{Product, Scraper};
use fantoccini::{wd::Capabilities, ClientBuilder, Locator};
use rust_decimal::prelude::*;
use scraper::{selectable::Selectable, Html, Selector};
use serde_json::json;

struct Barbora {
    url: Url,
}

impl Barbora {
    fn parse_html(&self, html: &str) -> anyhow::Result<Vec<Product>> {
        let document = Html::parse_document(html);

        let product_card_sel = Selector::parse("li").unwrap();

        let product_cards = document.select(&product_card_sel);

        let price_sel =
            Selector::parse(r#"div[id^="fti-product-price-category-page-"] div div:last-child"#)
                .unwrap();
        let product_title_sel =
            Selector::parse(r#"span[id^="fti-product-title-category-page-"]"#).unwrap();

        let products: Vec<Product> = product_cards
            .filter_map(|card| {
                let product_name: &str = card
                    .select(&product_title_sel)
                    .next()?
                    .text()
                    .nth(0)?
                    .trim_matches('\n')
                    .trim();
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

#[async_trait]
impl Scraper for Barbora {
    #[instrument(skip(self))]
    async fn run(&self) -> anyhow::Result<Vec<Product>> {
        let mut caps = Capabilities::new();
        caps.insert(
            "moz:firefoxOptions".to_string(),
            json!({
            "args": ["-headless"]
            }),
        );

        let c = ClientBuilder::native()
            .capabilities(caps)
            .connect("http://localhost:4444")
            .await
            .expect("failed to connect to WebDriver");
        c.goto(self.url.as_ref()).await?;

        let grid = c
            .wait()
            .for_element(Locator::Css("#category-page-results-placeholder ul"))
            .await?;

        let html = grid.html(true).await?;
        c.close().await?;

        self.parse_html(&html)
    }
}

#[cfg(test)]
mod tests {
    use crate::logger::init_tracing;

    use super::*;

    #[tokio::test]
    async fn test_barbora_parsing() {
        init_tracing();
        let base_url = Url::parse("https://barbora.ee/").expect("Failed to parse base_url");

        let path_to_beef = "/liha-kala-valmistoit/liha/veis-ja-muu-varske-liha".to_string();
        let url = base_url.join(&path_to_beef).unwrap();

        let b = Barbora { url };

        let products = b.run().await.unwrap();
        dbg!(products);
    }
}
