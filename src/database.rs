use rusqlite::{Connection, Result};

use crate::runtime_err::RunTimeError;
pub fn get_conn() -> Result<Connection, RunTimeError> {
    let conn = Connection::open("data/tiktok.db").map_err(|e| RunTimeError::DatabaseError(e))?;
    conn.busy_timeout(std::time::Duration::from_secs(5))
        .map_err(|e| RunTimeError::DatabaseError(e))?;
    let journal_mode: String = conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))?;
    let mmap_size: i64 = conn.query_row("PRAGMA mmap_size=30000000000;", [], |row| row.get(0))?;
    log::debug!("journal_mode: {} mmap_size: {}", journal_mode, mmap_size);
    conn.execute("PRAGMA cache_size=-64000;", [])?; // Set cache size to 64000 KB
    conn.execute("PRAGMA synchronous=NORMAL;", [])?; // Enable concurrent write
    conn.execute("PRAGMA temp_store=MEMORY;", [])?; // Use memory to store temporary data
    conn.execute("PRAGMA page_size=32768;", [])?; // Set page size to 32KB
    return Ok(conn);
}
pub fn add_column(table: &str, column_name: &str, ddl: &str) -> Result<(), RunTimeError> {
    let conn = get_conn()?;
    // Check if the new column exists
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({})", table))?;
    let rows = stmt.query_map([], |row| Ok(row.get::<_, String>(1)?))?;

    let mut column_exists = false;
    for row in rows {
        if let Ok(name) = row {
            if name == column_name {
                column_exists = true;
                break;
            }
        }
    }

    // If the new column does not exist, add it
    if !column_exists {
        conn.execute(ddl, rusqlite::params![])?;
    }
    Ok(())
}
pub fn create_databases() -> Result<(), RunTimeError> {
    let conn = get_conn()?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS `group` (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            title TEXT DEFAULT NULL,
            auto_publish INTEGER NOT NULL DEFAULT 1,
            publish_start_time TEXT DEFAULT '02:10',
            auto_train INTEGER NOT NULL DEFAULT 1,
            publish_type INTEGER NOT NULL DEFAULT 1,
            product_link TEXT DEFAULT NULL,
            train_start_time TEXT DEFAULT '20:10,20:30,21:10,21:30'
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS device (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            serial TEXT NOT NULL,
            forward_port INTEGER NOT NULL DEFAULT 0,
            online INTEGER NOT NULL DEFAULT 0,
            agent_ip TEXT NOT NULL,
            master_ip TEXT NOT NULL,
            init INTEGER NOT NULL DEFAULT 0,
            update_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS account (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER DEFAULT 0,
            email TEXT NOT NULL,
            pwd TEXT NOT NULL,
            fans INTEGER NOT NULL,
            shop_creator INTEGER NOT NULL DEFAULT 0,
            device TEXT DEFAULT NULL,
            username TEXT DEFAULT NULL,
            earnings INTEGER NOT NULL DEFAULT 0,
            today_sales INTEGER NOT NULL DEFAULT 0,
            today_sold_items INTEGER NOT NULL DEFAULT 0,
            today_orders INTEGER NOT NULL DEFAULT 0,
            register_time TEXT DEFAULT CURRENT_TIMESTAMP,
            last_login_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    add_column(
        "account",
        "username",
        "ALTER TABLE account ADD COLUMN `username` TEXT DEFAULT NULL",
    )?;
    add_column(
        "account",
        "earnings",
        "ALTER TABLE account ADD COLUMN `earnings` INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column(
        "account",
        "today_sales",
        "ALTER TABLE account ADD COLUMN `today_sales` INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column(
        "account",
        "today_sold_items",
        "ALTER TABLE account ADD COLUMN `today_sold_items` INTEGER NOT NULL DEFAULT 0",
    )?;
    add_column(
        "account",
        "today_orders",
        "ALTER TABLE account ADD COLUMN `today_orders` INTEGER NOT NULL DEFAULT 0",
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS material (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER  DEFAULT 0,
            name TEXT NOT NULL,
            md5 TEXT NOT NULL,
            used INTEGER NOT NULL DEFAULT 0,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS avatar (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS publish_job (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER  DEFAULT 0,
            material TEXT NOT NULL,
            account_id INTEGER NOT NULL DEFAULT 0,
            title TEXT DEFAULT NULL,
            status INTEGER NOT NULL DEFAULT 0,
            start_time TEXT DEFAULT CURRENT_TIMESTAMP,
            end_time TEXT DEFAULT CURRENT_TIMESTAMP,
            publish_type INTEGER NOT NULL DEFAULT 1,
            product_link TEXT DEFAULT NULL,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    add_column(
        "publish_job",
        "account_id",
        "ALTER TABLE publish_job ADD COLUMN `account_id` INTEGER NOT NULL DEFAULT 0",
    )?;
    //update publish_job set account = (select id from account where email = publish_job.account) where account like '%.com';
    //update publish_job set account = (select id from account where username = publish_job.account) where account like '@%';
    //update publish_job set account_id=account;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS train_job (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER  DEFAULT 0,
            account_id INTEGER NOT NULL DEFAULT 0,
            click INTEGER  DEFAULT 0,
            follow INTEGER  DEFAULT 0,
            favorites INTEGER  DEFAULT 0,
            status INTEGER NOT NULL DEFAULT 0,
            start_time TEXT DEFAULT CURRENT_TIMESTAMP,
            end_time TEXT DEFAULT CURRENT_TIMESTAMP,
            create_time TEXT DEFAULT CURRENT_TIMESTAMP
          );",
        (),
    )?;
    add_column(
        "train_job",
        "account_id",
        "ALTER TABLE train_job ADD COLUMN `account_id` INTEGER NOT NULL DEFAULT 0",
    )?;
    //update train_job set account = (select id from account where email = train_job.account) where account like '%.com';
    //update train_job set account = (select id from account where username = train_job.account) where account like '@%';
    //update train_job set account_id=account;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS dialog_watcher (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        conditions TEXT NOT NULL,
        action TEXT NOT NULL,
        status INTEGER NOT NULL DEFAULT 0,
        create_time TEXT DEFAULT CURRENT_TIMESTAMP
      );",
        (),
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS music (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        release_name TEXT NOT NULL,
        artist_name TEXT NOT NULL,
        create_time TEXT DEFAULT CURRENT_TIMESTAMP
      );",
        (),
    )?;
    Ok(())
}
