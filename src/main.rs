#![allow(unused)]
use std::io::Write;

use sqlx::{SqliteConnection, Connection, Row};
use futures::TryStreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut conn = SqliteConnection::connect(&std::env::var("DATABASE_URL")?).await?;
    let mut results_stream = sqlx::query("SELECT * FROM response_images").fetch(&mut conn);
    let mut i = 0;

    let mut urls = Vec::new();
    while let Some(row) = results_stream.try_next().await? {
        let image: Vec<u8> = row.get("url");
        let s = std::str::from_utf8(&image)?;
        let s = s.to_owned();
        println!("url: {}", s);
        urls.push(s.clone());

        if !s.contains("discordapp") {
            continue;
        }

        let extension = s.split('.').last().unwrap();
        let bytes = reqwest::get(&s).await?.bytes().await?;

        let mut file = std::fs::File::create(format!("images/{}.{}", i, extension))?;
        println!("writing to file");
        file.write_all(&bytes)?;
        i += 1;
    }

    let mut file = std::fs::File::create("urls.txt")?;
    for url in urls {
        println!("writing url to disk");
        file.write_all(url.as_bytes())?;
        file.write_all(b"\n")?;
    }

    Ok(())
}
