use anyhow::{bail, Result};
use std::fs::File;
use std::io::prelude::*;

fn main() -> Result<()> {
    // Parse arguments
    let args = std::env::args().collect::<Vec<_>>();
    match args.len() {
        0 | 1 => bail!("Missing <database path> and <command>"),
        2 => bail!("Missing <command>"),
        _ => {}
    }

    // Parse command and act accordingly
    let command = &args[2];
    match command.as_str() {
        ".dbinfo" => {
            let mut file = File::open(&args[1])?;
            let mut file_header = [0; 100];
            file.read_exact(&mut file_header)?;

            let page_size = u16::from_be_bytes([file_header[16], file_header[17]]);

            println!("database page size: {page_size}");

            // https://www.sqlite.org/fileformat.html#b_tree_pages
            // The b-tree page header is 8 bytes in size for leaf pages and 12 bytes for interior pages.
            let mut page_header = [0; 12];
            file.read_exact(&mut page_header)?;

            assert_eq!(0xd, page_header[0]);

            let num_tables = u16::from_be_bytes([page_header[3], page_header[4]]);
            println!("number of tables: {num_tables}");
        }
        _ => bail!("Missing or invalid command passed: {}", command),
    }

    Ok(())
}
