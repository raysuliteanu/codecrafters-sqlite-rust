#[derive(Debug)]
pub struct TableInfo {}

/*
Table B-Tree Leaf Cell (header 0x0d):

        A varint which is the total number of bytes of payload, including any overflow
        A varint which is the integer key, a.k.a. "rowid"
        The initial portion of the payload that does not spill to overflow pages.
        A 4-byte big-endian integer page number for the first page of the overflow page list - omitted if all payload fits on the b-tree page.
*/
enum Cell {
    TableLeaf {
        payload_len: u16,
        row_id: u16,
        payload: Option<Record>,
    },
}

struct Record {}
