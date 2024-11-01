use std::io::{BufReader, Read};

use crate::table::TableInfo;

#[derive(PartialEq, Debug)]
pub enum PageType {
    InternalIndex,
    InternalTable,
    LeafIndex,
    LeafTable,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct PageInfo {
    page_type: PageType,
    first_freeblock_idx: u16,
    pub num_cells: u16,
    cell_start_idx: u16,
    num_fragments: u8,
    rightmost_pointer: Option<u32>, // only PageType::Internal*
    tables: Vec<TableInfo>,
}

impl PageInfo {
    pub fn read_page(reader: &mut BufReader<impl Read>) -> Result<PageInfo, anyhow::Error> {
        // https://www.sqlite.org/fileformat.html#b_tree_pages
        // The b-tree page header is 8 bytes in size for leaf pages and 12 bytes for interior pages.

        /* The one-byte flag at offset 0 indicating the b-tree page type.
            A value of 2 (0x02) means the page is an interior index b-tree page.
            A value of 5 (0x05) means the page is an interior table b-tree page.
            A value of 10 (0x0a) means the page is a leaf index b-tree page.
            A value of 13 (0x0d) means the page is a leaf table b-tree page.

        Any other value for the b-tree page type is an error.!
        */
        let mut buf = [0u8; 1];
        reader.read_exact(&mut buf)?;
        let page_ind = u8::from_be_bytes(buf);
        let page_type = match page_ind {
            0x02 => PageType::InternalIndex,
            0x05 => PageType::InternalTable,
            0x0a => PageType::LeafIndex,
            0x0d => PageType::LeafTable,
            _e => panic!("invalid page type {_e}"),
        };

        let mut page_header = [0u8; 7];
        reader.read_exact(&mut page_header)?;

        let mut page_info = PageInfo {
            page_type,
            first_freeblock_idx: u16::from_be_bytes([page_header[0], page_header[1]]),
            num_cells: u16::from_be_bytes([page_header[2], page_header[3]]),
            cell_start_idx: u16::from_be_bytes([page_header[4], page_header[5]]),
            num_fragments: u8::from_be_bytes([page_header[6]]),
            rightmost_pointer: None,
            tables: vec![],
        };

        if page_info.page_type == PageType::InternalIndex
            || page_info.page_type == PageType::InternalTable
        {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf)?;
            page_info.rightmost_pointer = Some(u32::from_be_bytes(buf));
        }

        Ok(page_info)
    }

    fn read_cell(&mut self) {}
}
