mod barbora;
mod rimi;
mod selver;
use async_trait::async_trait;
use barbora::Barbora;
use rimi::Rimi;
use rust_decimal::Decimal;
use selver::Selver;
use sqlx::SqlitePool;
use sqlx::Row;
use reqwest::Url;

#[derive(Debug)]
pub struct PriceEur(Decimal);

#[derive(Debug)]
pub struct Product {
    name: String,
    price: PriceEur,
}

#[async_trait]
pub trait ScraperImpl {
    async fn run(&self) -> anyhow::Result<Vec<Product>>;
}

pub struct Scraper {
    id: i64,
    name: String,
    url: String,
    inner: Box<dyn ScraperImpl>
}

impl Scraper {
    pub async fn run(&self) -> anyhow::Result<Vec<Product>> {
        self.inner.run().await
    }
}

async fn get_scrapers(pool: &SqlitePool) -> Result<Vec<Scraper>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
            select id, name, scrape_url, scrape_impl
            from shop
            "#,
    )
    .fetch_all(pool)
    .await?;

    let mut scrapers = vec![];

    for row in rows {
        let url : String = row.get(2);
        let url = Url::parse(&url).unwrap();
        let imp : Box<dyn ScraperImpl> = 
        match row.get(3) {
            "rimi" => Box::new(Rimi { url }),
            "selver" => Box::new(Selver { url }),
            "barbora" => Box::new(Barbora { url }),
            _ => panic!("Unknown scrape implementation")
        };

        let s = Scraper {
            id: row.get(0),
            name: row.get(1),
            url: row.get(2),
            inner: imp
        };
        scrapers.push(s);
    }


    Ok(scrapers)
}
