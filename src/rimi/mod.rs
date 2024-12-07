use fantoccini::ClientBuilder;
use reqwest::Url;
use scraper::{selectable::Selectable, Html, Selector};
use tokio::process::Command;

struct Config {
    base_url: Url,
    path_to_beef: String,
}

async fn run() {
    //let mut cmd = Command::new("geckodriver");
    //let gd = cmd.spawn().expect("failed to run geckodriver");

    let base_url = Url::parse("https://www.rimi.ee/").expect("Failed to parse base_url");

    let path_to_beef =
        "https://www.rimi.ee/epood/ee/tooted/liha--ja-kalatooted/veise--lamba--ja-ulukiliha/c/SH-8-21".to_string();

    let sample_confg = Config {
        base_url,
        path_to_beef,
    };

    let url = sample_confg
        .base_url
        .join(&sample_confg.path_to_beef)
        .unwrap();

    //let c = ClientBuilder::native().connect("http://localhost:4444").await.expect("failed to connect to WebDriver");
    //c.goto(url.as_ref()).await.expect("failed to navigate");

    let html = reqwest::get(url).await.unwrap().text().await.unwrap();
    //let html = c.source().await.expect("failed to get html");

    let document = Html::parse_document(&html);

    let product_listing_sel = Selector::parse(".product-grid").unwrap();

    let product_card_sel = Selector::parse(".product-grid__item").unwrap();

    let product = document.select(&product_listing_sel).next().unwrap();

    let product_cards = product.select(&product_card_sel);

    let price_sel = Selector::parse(".card__price-per").unwrap();
    let product_title_sel = Selector::parse(".card__name").unwrap();

    for card in product_cards {
        println!("----------");
        let prices: Vec<&str> = card.select(&price_sel).next().unwrap().text().collect();
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
