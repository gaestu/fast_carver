use std::path::Path;

use anyhow::Result;
use rusqlite::{Connection, OpenFlags};

use crate::parsers::browser::BrowserHistoryRecord;

pub fn extract_browser_history(
    path: &Path,
    run_id: &str,
    source_relative: &str,
) -> Result<Vec<BrowserHistoryRecord>> {
    let mut out = Vec::new();
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;

    if has_table(&conn, "urls")? {
        if has_table(&conn, "visits")? {
            if let Ok(records) = extract_chrome_visits(&conn, run_id, source_relative) {
                out.extend(records);
            }
        } else if let Ok(records) = extract_chrome_history(&conn, run_id, source_relative) {
            out.extend(records);
        }
    }

    if has_table(&conn, "moz_places")? {
        if has_table(&conn, "moz_historyvisits")? {
            if let Ok(records) = extract_firefox_visits(&conn, run_id, source_relative) {
                out.extend(records);
            }
        } else if let Ok(records) = extract_firefox_history(&conn, run_id, source_relative) {
            out.extend(records);
        }
    }

    Ok(out)
}

fn has_table(conn: &Connection, name: &str) -> Result<bool> {
    let mut stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")?;
    let mut rows = stmt.query([name])?;
    Ok(rows.next()?.is_some())
}

fn extract_chrome_history(
    conn: &Connection,
    run_id: &str,
    source_relative: &str,
) -> Result<Vec<BrowserHistoryRecord>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare("SELECT url, title, last_visit_time FROM urls")?;
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: Option<String> = row.get(1)?;
        let last_visit_time: Option<i64> = row.get(2)?;
        Ok((url, title, last_visit_time))
    })?;

    for row in rows {
        let (url, title, last_visit_time) = row?;
        let visit_time = last_visit_time.and_then(webkit_timestamp_to_datetime);
        out.push(BrowserHistoryRecord {
            run_id: run_id.to_string(),
            browser: "chrome".to_string(),
            profile: "Default".to_string(),
            url,
            title,
            visit_time,
            visit_source: None,
            source_file: source_relative.into(),
        });
    }

    Ok(out)
}

fn extract_chrome_visits(
    conn: &Connection,
    run_id: &str,
    source_relative: &str,
) -> Result<Vec<BrowserHistoryRecord>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT urls.url, urls.title, visits.visit_time, visits.transition FROM visits JOIN urls ON visits.url = urls.id",
    )?;
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: Option<String> = row.get(1)?;
        let visit_time: Option<i64> = row.get(2)?;
        let transition: Option<i64> = row.get(3)?;
        Ok((url, title, visit_time, transition))
    })?;

    for row in rows {
        let (url, title, visit_time, transition) = row?;
        let visit_time = visit_time.and_then(webkit_timestamp_to_datetime);
        let visit_source = transition.map(chrome_transition_label).map(|s| s.to_string());
        out.push(BrowserHistoryRecord {
            run_id: run_id.to_string(),
            browser: "chrome".to_string(),
            profile: "Default".to_string(),
            url,
            title,
            visit_time,
            visit_source,
            source_file: source_relative.into(),
        });
    }

    Ok(out)
}

fn extract_firefox_history(
    conn: &Connection,
    run_id: &str,
    source_relative: &str,
) -> Result<Vec<BrowserHistoryRecord>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare("SELECT url, title, last_visit_date FROM moz_places")?;
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: Option<String> = row.get(1)?;
        let last_visit_date: Option<i64> = row.get(2)?;
        Ok((url, title, last_visit_date))
    })?;

    for row in rows {
        let (url, title, last_visit_date) = row?;
        let visit_time = last_visit_date.and_then(unix_micro_to_datetime);
        out.push(BrowserHistoryRecord {
            run_id: run_id.to_string(),
            browser: "firefox".to_string(),
            profile: "Default".to_string(),
            url,
            title,
            visit_time,
            visit_source: None,
            source_file: source_relative.into(),
        });
    }

    Ok(out)
}

fn extract_firefox_visits(
    conn: &Connection,
    run_id: &str,
    source_relative: &str,
) -> Result<Vec<BrowserHistoryRecord>> {
    let mut out = Vec::new();
    let mut stmt = conn.prepare(
        "SELECT moz_places.url, moz_places.title, moz_historyvisits.visit_date, moz_historyvisits.visit_type \
         FROM moz_historyvisits JOIN moz_places ON moz_historyvisits.place_id = moz_places.id",
    )?;
    let rows = stmt.query_map([], |row| {
        let url: String = row.get(0)?;
        let title: Option<String> = row.get(1)?;
        let visit_date: Option<i64> = row.get(2)?;
        let visit_type: Option<i64> = row.get(3)?;
        Ok((url, title, visit_date, visit_type))
    })?;

    for row in rows {
        let (url, title, visit_date, visit_type) = row?;
        let visit_time = visit_date.and_then(unix_micro_to_datetime);
        let visit_source = visit_type.map(firefox_visit_label).map(|s| s.to_string());
        out.push(BrowserHistoryRecord {
            run_id: run_id.to_string(),
            browser: "firefox".to_string(),
            profile: "Default".to_string(),
            url,
            title,
            visit_time,
            visit_source,
            source_file: source_relative.into(),
        });
    }

    Ok(out)
}

fn chrome_transition_label(transition: i64) -> &'static str {
    match transition & 0xFF {
        0 => "link",
        1 => "typed",
        2 => "auto_bookmark",
        3 => "auto_subframe",
        4 => "manual_subframe",
        5 => "generated",
        6 => "auto_toplevel",
        7 => "form_submit",
        8 => "reload",
        9 => "keyword",
        10 => "keyword_generated",
        _ => "other",
    }
}

fn firefox_visit_label(visit_type: i64) -> &'static str {
    match visit_type {
        1 => "link",
        2 => "typed",
        3 => "bookmark",
        4 => "embed",
        5 => "redirect_permanent",
        6 => "redirect_temporary",
        7 => "download",
        8 => "framed_link",
        _ => "other",
    }
}

fn webkit_timestamp_to_datetime(microseconds: i64) -> Option<chrono::NaiveDateTime> {
    if microseconds <= 0 {
        return None;
    }
    let unix_offset_seconds = 11_644_473_600i64;
    let secs = microseconds / 1_000_000 - unix_offset_seconds;
    if secs < 0 {
        return None;
    }
    let nsecs = ((microseconds % 1_000_000).abs() as u32) * 1000;
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
}

fn unix_micro_to_datetime(microseconds: i64) -> Option<chrono::NaiveDateTime> {
    if microseconds <= 0 {
        return None;
    }
    let secs = microseconds / 1_000_000;
    let nsecs = ((microseconds % 1_000_000).abs() as u32) * 1000;
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, nsecs).map(|dt| dt.naive_utc())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn extracts_chrome_history() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("History");
        let conn = Connection::open(&path).expect("conn");
        conn.execute(
            "CREATE TABLE urls (id INTEGER PRIMARY KEY, url TEXT, title TEXT, last_visit_time INTEGER)",
            [],
        )
        .expect("create");
        conn.execute(
            "INSERT INTO urls (url, title, last_visit_time) VALUES (?1, ?2, ?3)",
            ("https://example.com", "Example", 13_303_449_600_000_000i64),
        )
        .expect("insert");
        drop(conn);

        let records = extract_browser_history(&path, "run1", "sqlite/history.sqlite").expect("history");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].browser, "chrome");
        assert_eq!(records[0].url, "https://example.com");
    }

    #[test]
    fn extracts_chrome_visits() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("History");
        let conn = Connection::open(&path).expect("conn");
        conn.execute(
            "CREATE TABLE urls (id INTEGER PRIMARY KEY, url TEXT, title TEXT)",
            [],
        )
        .expect("create urls");
        conn.execute(
            "CREATE TABLE visits (id INTEGER PRIMARY KEY, url INTEGER, visit_time INTEGER, transition INTEGER)",
            [],
        )
        .expect("create visits");
        conn.execute(
            "INSERT INTO urls (id, url, title) VALUES (1, ?1, ?2)",
            ("https://example.com", "Example"),
        )
        .expect("insert url");
        conn.execute(
            "INSERT INTO visits (url, visit_time, transition) VALUES (1, ?1, 1)",
            (13_303_449_600_000_000i64,),
        )
        .expect("insert visit");
        drop(conn);

        let records = extract_browser_history(&path, "run1", "sqlite/history.sqlite").expect("history");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].browser, "chrome");
        assert_eq!(records[0].visit_source.as_deref(), Some("typed"));
    }

    #[test]
    fn extracts_firefox_visits() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("places.sqlite");
        let conn = Connection::open(&path).expect("conn");
        conn.execute(
            "CREATE TABLE moz_places (id INTEGER PRIMARY KEY, url TEXT, title TEXT)",
            [],
        )
        .expect("create places");
        conn.execute(
            "CREATE TABLE moz_historyvisits (id INTEGER PRIMARY KEY, place_id INTEGER, visit_date INTEGER, visit_type INTEGER)",
            [],
        )
        .expect("create visits");
        conn.execute(
            "INSERT INTO moz_places (id, url, title) VALUES (1, ?1, ?2)",
            ("https://example.com", "Example"),
        )
        .expect("insert place");
        conn.execute(
            "INSERT INTO moz_historyvisits (place_id, visit_date, visit_type) VALUES (1, ?1, 2)",
            (1_700_000_000_000_000i64,),
        )
        .expect("insert visit");
        drop(conn);

        let records = extract_browser_history(&path, "run1", "sqlite/history.sqlite").expect("history");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].browser, "firefox");
        assert_eq!(records[0].visit_source.as_deref(), Some("typed"));
    }
}
