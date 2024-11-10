use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::BufReader;

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
    let mut file_header = [0; 100];
    let db_info = db::open_db(&mut reader, &mut file_header).context("open_db")?;
    let page_info = db_info.read_page(&mut reader, Some(file_header))?;

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            println!("database page size: {}", db_info.page_size);
            println!("number of tables: {}", page_info.num_cells);
        }
        ".tables" => page_info.tables.iter().for_each(|t| println!("{}", t.name)),
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
