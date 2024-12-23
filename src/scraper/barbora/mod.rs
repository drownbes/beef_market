use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use reqwest::Url;
use crate::scraper::PriceEur;

use super::{Product, Scraper};
use fantoccini::{wd::Capabilities, ClientBuilder, Locator};
use scraper::{selectable::Selectable, Html, Selector};
use serde_json::json;
use rust_decimal::prelude::*;



struct Barbora {
    url: Url
}

impl Barbora {
    fn parse_html(&self, html: &str) -> anyhow::Result<Vec<Product>> {
        let document = Html::parse_document(html);

        let product_card_sel = Selector::parse("li").unwrap();

        let product_cards = document.select(&product_card_sel);

        let price_sel = Selector::parse(r#"div[id^="fti-product-price-category-page-"] div div:last-child"#).unwrap();
        let product_title_sel = Selector::parse(r#"span[id^="fti-product-title-category-page-"]"#).unwrap();

        let mut products = vec![];

        for card in product_cards {
            let p = card.select(&price_sel).next();
            if p.is_none() {
                continue;
            }
            let prices: Vec<&str> = p.unwrap().text().collect();
            let product_name: Vec<&str> = card
                .select(&product_title_sel)
                .next()
                .unwrap()
                .text()
                .collect();

            let name = product_name.first().unwrap().trim_matches('\n').trim();
            let price: String = prices.first().unwrap().trim().split('\n').collect();
            let price : String = price.strip_suffix("€/kg")
                .ok_or(anyhow!("failed to strip kg"))?.replace(",", ".");
            let price : Decimal = Decimal::from_str(&price).context("Failed to parse to decimal")?;
            println!("Product: {} with price: {}", name, price);
            products.push(Product {
                name: name.into(),
                price: PriceEur (price)
            });
            
        }

        Ok(products)
    }
}

#[async_trait]
impl Scraper for Barbora {
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
    use super::*;

    #[tokio::test]
    async fn test_barbora_parsing() {
        let base_url = Url::parse("https://barbora.ee/").expect("Failed to parse base_url");

        let path_to_beef = "/liha-kala-valmistoit/liha/veis-ja-muu-varske-liha".to_string();
        let url = base_url.join(&path_to_beef).unwrap();

        let b = Barbora { url };

        let products = b.run().await.unwrap();
        dbg!(products);
    }
}


