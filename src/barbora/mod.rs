use fantoccini::{wd::Capabilities, ClientBuilder, Locator};
use reqwest::Url;
use scraper::{selectable::Selectable, Html, Selector};
use serde_json::json;

struct Config {
    base_url: Url,
    path_to_beef: String,
}

async fn run() {
    let base_url = Url::parse("https://barbora.ee/").expect("Failed to parse base_url");

    let path_to_beef = "/liha-kala-valmistoit/liha/veis-ja-muu-varske-liha".to_string();

    let sample_confg = Config {
        base_url,
        path_to_beef,
    };

    let url = sample_confg
        .base_url
        .join(&sample_confg.path_to_beef)
        .unwrap();

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
    c.goto(url.as_ref()).await.expect("failed to navigate");

    let grid = c
        .wait()
        .for_element(Locator::Css("#category-page-results-placeholder ul"))
        .await
        .expect("Failed to wait for grid");

    let html = grid.html(true).await.expect("Failed to get");

    c.close().await.unwrap();

    //let html = reqwest::get(url).await.unwrap().text().await.unwrap();

    let document = Html::parse_document(&html);

    let product_card_sel = Selector::parse("li").unwrap();

    let product_cards = document.select(&product_card_sel);

    let price_sel =
        Selector::parse(r#"div[id^="fti-product-price-category-page-"] div div:last-child"#)
            .unwrap();
    let product_title_sel =
        Selector::parse(r#"span[id^="fti-product-title-category-page-"]"#).unwrap();

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

        let product_name = product_name.first().unwrap().trim_matches('\n').trim();
        let unit_price: String = prices.first().unwrap().trim().split('\n').collect();
        println!("Product: {} with price: {}", product_name, unit_price);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_name() {
        run().await;
    }
}
