mod barbora;
mod rimi;
mod selver;
use async_trait::async_trait;
use barbora::Barbora;
use reqwest::Url;
use rimi::Rimi;
use rust_decimal::prelude::*;
use rust_decimal::Decimal;
use selver::Selver;
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::sqlite::SqliteArgumentValue;
use sqlx::sqlite::SqliteTypeInfo;
use sqlx::Encode;
use sqlx::Row;
use sqlx::Sqlite;
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct PriceEur(pub Decimal);

impl sqlx::Type<Sqlite> for PriceEur {
    fn type_info() -> SqliteTypeInfo {
        <i64 as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q> Encode<'q, Sqlite> for PriceEur {
    fn encode_by_ref(
        &self,
        args: &mut Vec<SqliteArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        let price_int: i64 = self
            .0
            .checked_mul(Decimal::from(100))
            .ok_or("Overflow")?
            .to_i64()
            .ok_or("Cannot represent as i64")?;
        args.push(SqliteArgumentValue::Int64(price_int));
        Ok(IsNull::No)
    }
}

#[derive(Debug)]
pub struct Product {
    pub name: String,
    pub price: PriceEur,
}

#[async_trait]
pub trait ScraperImpl {
    async fn run(&self) -> anyhow::Result<Vec<Product>>;
}

pub struct Scraper {
    pub id: i64,
    pub name: String,
    inner: Box<dyn ScraperImpl>,
}

impl Scraper {
    pub async fn run(&self) -> anyhow::Result<Vec<Product>> {
        self.inner.run().await
    }
}

pub async fn get_scrapers(pool: &SqlitePool) -> Result<Vec<Scraper>, sqlx::Error> {
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
        let url: String = row.get(2);
        let url = Url::parse(&url).unwrap();
        let imp: Box<dyn ScraperImpl> = match row.get(3) {
            "rimi" => Box::new(Rimi { url }),
            "selver" => Box::new(Selver { url }),
            "barbora" => Box::new(Barbora { url }),
            _ => panic!("Unknown scrape implementation"),
        };

        let s = Scraper {
            id: row.get(0),
            name: row.get(1),
            inner: imp,
        };
        scrapers.push(s);
    }

    Ok(scrapers)
}
