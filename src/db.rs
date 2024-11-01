use anyhow::{Context, Result};
use std::io::{prelude::*, BufReader};

use crate::util;

pub fn open_db(reader: &mut BufReader<impl Read>) -> Result<DbInfo, anyhow::Error> {
    let mut file_header = [0; 100];
    reader.read_exact(&mut file_header).context("file_header")?;

    // SAFETY: per spec: https://www.sqlite.org/fileformat.html#magic_header_string
    let magic = unsafe { std::str::from_utf8_unchecked(&file_header[0..16]) };
    assert_eq!(magic, "SQLite format 3\0");

    let page_size = u16::from_be_bytes([file_header[16], file_header[17]]);
    let write_version = u8::from_be_bytes([file_header[18]]);
    let read_version = u8::from_be_bytes([file_header[19]]);
    let per_page_reserved_space = u8::from_be_bytes([file_header[20]]);

    // check but don't store these next values
    let max_embedded_payload = u8::from_be_bytes([file_header[21]]);
    assert_eq!(max_embedded_payload, 64);
    let min_embedded_payload = u8::from_be_bytes([file_header[22]]);
    assert_eq!(min_embedded_payload, 32);
    let leaf_payload = u8::from_be_bytes([file_header[23]]);
    assert_eq!(leaf_payload, 32);

    let mut file_header = file_header.as_slice();

    let file_change_counter = util::read_be_u32(&mut file_header);
    let in_header_db_size = util::read_be_u32(&mut file_header);
    let first_freelist_trunk_page = util::read_be_u32(&mut file_header);
    let num_freelist_pages = util::read_be_u32(&mut file_header);
    let schema_cookie = util::read_be_u32(&mut file_header);
    let schema_format = util::read_be_u32(&mut file_header);
    let default_page_cache_size = util::read_be_u32(&mut file_header);
    let largest_root_page = util::read_be_u32(&mut file_header);
    let text_encoding = util::read_be_u32(&mut file_header);
    let user_version = util::read_be_u32(&mut file_header);
    let inc_vacuum_mode = util::read_be_u32(&mut file_header);
    let app_id = util::read_be_u32(&mut file_header);

    // eat 20 bytes
    let _reserved = util::read_len(&mut file_header, 20);

    let version_valid_for = util::read_be_u32(&mut file_header);
    let version_num = util::read_be_u32(&mut file_header);

    Ok(DbInfo {
        page_size,
        write_version,
        read_version,
        per_page_reserved_space,
        file_change_counter,
        in_header_db_size,
        first_freelist_trunk_page,
        num_freelist_pages,
        schema_cookie,
        schema_format,
        default_page_cache_size,
        largest_root_page,
        text_encoding,
        user_version,
        inc_vacuum_mode,
        app_id,
        version_valid_for,
        version_num,
    })
}

// https://www.sqlite.org/fileformat.html#the_database_header
#[allow(dead_code)]
#[derive(Debug)]
pub struct DbInfo {
    /// The database page size in bytes. Must be a power of two between 512 and
    /// 32768 inclusive, or the value 1 representing a page size of 65536.
    pub page_size: u16,
    /// File format write version. 1 for legacy; 2 for WAL.
    write_version: u8,
    /// File format read version. 1 for legacy; 2 for WAL.
    read_version: u8,
    /// Bytes of unused "reserved" space at the end of each page. Usually 0.
    per_page_reserved_space: u8,
    /// File change counter.
    file_change_counter: u32,
    /// Size of the database file in pages. The "in-header database size".
    in_header_db_size: u32,
    /// Page number of the first freelist trunk page.
    first_freelist_trunk_page: u32,
    /// Total number of freelist pages.
    num_freelist_pages: u32,
    /// The schema cookie.
    schema_cookie: u32,
    /// The schema format number. Supported schema formats are 1, 2, 3, and 4.
    schema_format: u32,
    /// Default page cache size.
    default_page_cache_size: u32,
    /// The page number of the largest root b-tree page when in auto-vacuum or
    /// incremental-vacuum modes, or zero otherwise.
    largest_root_page: u32,
    /// The database text encoding. A value of 1 means UTF-8. A value of 2
    /// means UTF-16le. A value of 3 means UTF-16be.
    text_encoding: u32,
    /// The "user version" as read and set by the user_version pragma.
    /// https://www.sqlite.org/pragma.html#pragma_user_version
    user_version: u32,
    /// True (non-zero) for incremental-vacuum mode. False (zero) otherwise.
    inc_vacuum_mode: u32,
    /// The "Application ID" set by PRAGMA application_id
    /// https://www.sqlite.org/pragma.html#pragma_application_id
    app_id: u32,
    /// The version-valid-for number.
    /// https://www.sqlite.org/fileformat2.html#validfor
    version_valid_for: u32,
    /// https://www.sqlite.org/c3ref/c_source_id.html
    version_num: u32,
}
