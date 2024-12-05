use reqwest::Url;
use scraper::{selectable::Selectable, Html, Selector};

struct SelverConfig {
    base_url: Url,
    path_to_beef: String,
}

async fn run() {
    let base_url = Url::parse("https://www.selver.ee/").expect("Failed to parse base_url");

    let path_to_beef =
        "/liha-ja-kalatooted/veise-lamba-ja-ulukiliha?product_segment=4725".to_string();

    let sample_confg = SelverConfig {
        base_url,
        path_to_beef,
    };

    let url = sample_confg
        .base_url
        .join(&sample_confg.path_to_beef)
        .unwrap();

    let html = reqwest::get(url).await.unwrap().text().await.unwrap();

    let document = Html::parse_document(&html);

    let product_listing_sel = Selector::parse(".ProductListing").unwrap();

    let product_card_sel = Selector::parse(".ProductCard").unwrap();

    let product = document.select(&product_listing_sel).next().unwrap();

    let product_cards = product.select(&product_card_sel);

    let price_sel = Selector::parse(".ProductPrice__unit-price").unwrap();
    let product_title_sel = Selector::parse(".ProductCard__title").unwrap();

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
        let unit_price = prices.first().unwrap();
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
