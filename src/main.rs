use anyhow::{bail, Context, Result};
use page::PageInfo;
use std::fs::File;
use std::io::{prelude::*, BufReader};

mod db;
mod page;
mod table;
mod util;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    let file = File::open(&args[1])?;
    let mut reader = BufReader::new(&file);
    let db_info = db::open_db(&mut reader).context("open_db")?;

    let page_info = PageInfo::read_page(&mut reader)?;

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            println!("database page size: {}", db_info.page_size);
            println!("number of tables: {}", page_info.num_cells);
        }
        ".tables" => {}
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
