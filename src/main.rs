#![allow(unused)]
use std::io::Write;

use clap::Parser;
use sqlx::{SqliteConnection, Connection, Row};
use futures::TryStreamExt;

#[derive(Debug, Parser)]
#[clap(name = "basic")]
struct Args {
    #[arg(short, long, help = "The sqlite database file to read from")]
    file: String,
    #[arg(short, long, help = "The folder to write the images to")]
    output_file: String,
    #[arg(short, long, default_value = "urls.txt", help = "The file to write the urls to")]
    urls_file: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let db_url = String::from("sqlite://") + &args.file;
    let mut conn = SqliteConnection::connect(&db_url).await?;
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

        let mut file = std::fs::File::create(format!("{}/{}.{}", args.output_file, i, extension))?;
        println!("writing to file");
        file.write_all(&bytes)?;
        i += 1;
    }

    let mut file = std::fs::File::create(args.urls_file)?;
    for url in urls {
        println!("writing url to disk");
        file.write_all(url.as_bytes())?;
        file.write_all(b"\n")?;
    }

    Ok(())
}
