use crate::table::{ColumnType, TableInfo};
use crate::util;

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
    pub tables: Vec<TableInfo>,
}

impl PageInfo {
    pub fn read(buf: Vec<u8>, page_start: usize) -> PageInfo {
        // https://www.sqlite.org/fileformat.html#b_tree_pages
        // The b-tree page header is 8 bytes in size for leaf pages and 12 bytes for interior pages.

        /*
            The one-byte flag at offset 0 indicating the b-tree page type.
            A value of 2 (0x02) means the page is an interior index b-tree page.
            A value of 5 (0x05) means the page is an interior table b-tree page.
            A value of 10 (0x0a) means the page is a leaf index b-tree page.
            A value of 13 (0x0d) means the page is a leaf table b-tree page.

            Any other value for the b-tree page type is an error.!
        */
        let page_header = &buf[page_start..page_start + 8];
        let page_ind = u8::from_be_bytes([page_header[0]]);
        let page_type = match page_ind {
            0x02 => PageType::InternalIndex,
            0x05 => PageType::InternalTable,
            0x0a => PageType::LeafIndex,
            0x0d => PageType::LeafTable,
            _e => panic!("invalid page type {_e}"),
        };

        let mut page_info = PageInfo {
            page_type,
            first_freeblock_idx: u16::from_be_bytes([page_header[1], page_header[2]]),
            num_cells: u16::from_be_bytes([page_header[3], page_header[4]]),
            cell_start_idx: u16::from_be_bytes([page_header[5], page_header[6]]),
            num_fragments: u8::from_be_bytes([page_header[7]]),
            rightmost_pointer: None,
            tables: vec![],
        };

        let mut offset = 8usize;

        if page_info.page_type == PageType::InternalIndex
            || page_info.page_type == PageType::InternalTable
        {
            page_info.rightmost_pointer =
                Some(u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]));
            offset += 4;
        }

        page_info.read_cells(buf, page_start + offset);

        page_info
    }

    // buf is the full page content, so offset is where the cell ptrs start
    fn read_cells(&mut self, buf: Vec<u8>, offset: usize) -> Result<(), anyhow::Error> {
        let mut cell_ptrs = vec![];
        for i in 0..self.num_cells {
            cell_ptrs.push(u16::from_be_bytes([
                buf[offset + (i * 2) as usize],
                buf[offset + (i * 2 + 1) as usize],
            ]));
        }

        match self.page_type {
            PageType::LeafTable => {
                for cell_ptr in cell_ptrs {
                    println!("buf[{:#X}]={:#X}", cell_ptr, buf[cell_ptr as usize]);
                    let mut cell_ptr_offset = cell_ptr as usize;

                    let cell = TableLeaf::new(&buf, cell_ptr_offset)?;

                    // record_hdr_len is count of bytes including itself; since record_hdr_len is also
                    // a varint, the number of bytes it took up in buf is the most recent cnt
                    let limit = cell.header_len + cell_ptr_offset;
                    let mut col_types = Vec::new();
                    while cell_ptr_offset < limit {
                        let (type_cd, varint_sz) = util::varint_unsigned(&buf[cell_ptr_offset..])?;
                        col_types.push(type_cd);
                        cell_ptr_offset += varint_sz;
                    }
                    println!("col_types = {:#X?}", col_types);

                    // see https://www.sqlite.org/fileformat.html#record_format

                    let mut record = DbRecord::new();
                    let mut idx = cell_ptr as usize;
                    col_types.iter().for_each(|col_type| {
                        let (len, val) = PageInfo::read_column(*col_type, &buf, idx);
                        record.push(val);
                        idx += len;
                    });

                    self.tables.push(TableInfo::new(record)?);
                }
            }
            _ => todo!(),
        }

        Ok(())
    }

    fn read_column(col_type: u64, buf: &Vec<u8>, idx: usize) -> (usize, ColumnType) {
        match col_type {
            0 => (0, ColumnType::Null),
            1 => (1, ColumnType::Int8(i8::from_be_bytes([buf[idx]]))),
            2 => (
                2,
                ColumnType::Int16(i16::from_be_bytes([buf[idx], buf[idx + 1]])),
            ),
            3 => (
                3,
                ColumnType::Int24(i32::from_be_bytes([
                    0,
                    buf[idx],
                    buf[idx + 1],
                    buf[idx + 2],
                ])),
            ),
            4 => (
                4,
                ColumnType::Int32(i32::from_be_bytes([
                    buf[idx],
                    buf[idx + 1],
                    buf[idx + 2],
                    buf[idx + 3],
                ])),
            ),
            5 => (
                6,
                ColumnType::Int48(i64::from_be_bytes([
                    0,
                    buf[idx],
                    buf[idx + 1],
                    buf[idx + 2],
                    buf[idx + 3],
                    buf[idx + 4],
                    buf[idx + 5],
                    buf[idx + 6],
                ])),
            ),
            6 => (
                8,
                ColumnType::Int64(i64::from_be_bytes([
                    buf[idx],
                    buf[idx + 1],
                    buf[idx + 2],
                    buf[idx + 3],
                    buf[idx + 4],
                    buf[idx + 5],
                    buf[idx + 6],
                    buf[idx + 7],
                ])),
            ),
            7 => (
                8,
                ColumnType::Float(f64::from_be_bytes([
                    buf[idx],
                    buf[idx + 1],
                    buf[idx + 2],
                    buf[idx + 3],
                    buf[idx + 4],
                    buf[idx + 5],
                    buf[idx + 6],
                    buf[idx + 7],
                ])),
            ),
            8 => (0, ColumnType::False),
            9 => (0, ColumnType::True),
            10 | 11 => unimplemented!("reserved for future use"),
            n if n / 2 == 0 => {
                let len = (n - 12 / 2) as usize;
                let mut blob = &buf[idx..];
                (
                    len,
                    ColumnType::Blob(Vec::from(util::read_len(&mut blob, len))),
                )
            }
            n if n / 2 != 0 => {
                let len = (n - 12 / 2) as usize;
                let data = &buf[idx..(idx + len)];
                (
                    len,
                    ColumnType::String(
                        std::str::from_utf8(data).expect("valid string").to_string(),
                    ),
                )
            }
            _ => {
                panic!("invalid column type");
            }
        }
    }
}

/*
Table B-Tree Leaf Cell (header 0x0d):

        A varint which is the total number of bytes of payload, including any overflow
        A varint which is the integer key, a.k.a. "rowid"
        The initial portion of the payload that does not spill to overflow pages.
        A 4-byte big-endian integer page number for the first page of the overflow page list - omitted if all payload fits on the b-tree page.
*/
struct TableLeaf {
    header_len: usize,
    payload_len: usize,
    row_id: u64,
    payload: Option<DbRecord>,
}

impl TableLeaf {
    fn new(buf: &Vec<u8>, idx: usize) -> Result<TableLeaf, anyhow::Error> {
        let mut cell_ptr_offset = idx;
        let (payload_len, cnt) = util::varint_unsigned(&buf[cell_ptr_offset..])?;
        cell_ptr_offset += cnt;
        let (row_id, cnt) = util::varint_unsigned(&buf[cell_ptr_offset..])?;
        cell_ptr_offset += cnt;
        let (record_hdr_len, cnt) = util::varint_unsigned(&buf[cell_ptr_offset..])?;
        let header_len = record_hdr_len as usize - cnt;

        Ok(TableLeaf {
            header_len,
            payload_len: payload_len as usize,
            row_id,
            payload: None,
        })
    }
}

pub(crate) type DbRecord = Vec<ColumnType>;
