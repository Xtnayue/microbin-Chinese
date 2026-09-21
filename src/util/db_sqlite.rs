use bytesize::ByteSize;
use rusqlite::{params, Connection};

use crate::pasta::{PastaFile, ReadAccess, WriteAccess};
use crate::{args::ARGS, Pasta};

/// 当前的表结构。三项权限各自独立成列：
/// `read_access`（read 权限）、`write_access`（write 权限）、`listed`（list 权限）。
const CREATE_TABLE_SQL: &str = "
    CREATE TABLE IF NOT EXISTS pasta (
        id INTEGER PRIMARY KEY,
        content TEXT NOT NULL,
        file_name TEXT,
        file_size INTEGER,
        extension TEXT NOT NULL,
        read_access INTEGER NOT NULL,
        write_access INTEGER NOT NULL,
        listed INTEGER NOT NULL,
        read_key TEXT,
        write_key TEXT,
        editable INTEGER NOT NULL,
        created INTEGER NOT NULL,
        expiration INTEGER NOT NULL,
        last_read INTEGER NOT NULL,
        read_count INTEGER NOT NULL,
        burn_after_reads INTEGER NOT NULL,
        attachments TEXT,
        pasta_type TEXT NOT NULL
    );";

/// 显式列出列名，避免依赖列顺序（旧库可能残留已废弃的列）。
const SELECT_COLUMNS: &str = "
    id, content, file_name, file_size, extension,
    read_access, write_access, listed, read_key, write_key,
    editable, created, expiration, last_read, read_count,
    burn_after_reads, attachments, pasta_type";

const INSERT_SQL: &str = "
    INSERT INTO pasta (
        id, content, file_name, file_size, extension,
        read_access, write_access, listed, read_key, write_key,
        editable, created, expiration, last_read, read_count,
        burn_after_reads, attachments, pasta_type
    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)";

const UPDATE_SQL: &str = "
    UPDATE pasta SET
        content = ?2,
        file_name = ?3,
        file_size = ?4,
        extension = ?5,
        read_access = ?6,
        write_access = ?7,
        listed = ?8,
        read_key = ?9,
        write_key = ?10,
        editable = ?11,
        created = ?12,
        expiration = ?13,
        last_read = ?14,
        read_count = ?15,
        burn_after_reads = ?16,
        attachments = ?17,
        pasta_type = ?18
    WHERE id = ?1;";

fn open() -> Connection {
    Connection::open(format!("{}/database.sqlite", ARGS.data_dir))
        .expect("Failed to open SQLite database!")
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
    let Ok(mut stmt) = conn.prepare(&format!("PRAGMA table_info({table})")) else {
        return false;
    };
    let Ok(mut rows) = stmt.query([]) else {
        return false;
    };
    while let Ok(Some(row)) = rows.next() {
        if let Ok(name) = row.get::<_, String>(1) {
            if name == column {
                return true;
            }
        }
    }
    false
}

/// 建表，并把旧的「隐私分档」表结构就地迁移到新的权限列。
///
/// 迁移是幂等的：只有检测到缺少 `read_access` 列时才执行。
fn ensure_schema(conn: &Connection) {
    conn.execute(CREATE_TABLE_SQL, params![])
        .expect("Failed to create SQLite table for Pasta!");

    if column_exists(conn, "pasta", "read_access") {
        return;
    }

    // 旧库：补齐新列（SQLite 的 ADD COLUMN 在 NOT NULL 时必须带默认值）
    let _ = conn.execute("ALTER TABLE pasta ADD COLUMN attachments TEXT", params![]);
    let _ = conn.execute(
        "ALTER TABLE pasta ADD COLUMN read_access INTEGER NOT NULL DEFAULT 1",
        params![],
    );
    let _ = conn.execute(
        "ALTER TABLE pasta ADD COLUMN write_access INTEGER NOT NULL DEFAULT 2",
        params![],
    );
    let _ = conn.execute(
        "ALTER TABLE pasta ADD COLUMN listed INTEGER NOT NULL DEFAULT 0",
        params![],
    );
    let _ = conn.execute("ALTER TABLE pasta ADD COLUMN read_key TEXT", params![]);
    let _ = conn.execute("ALTER TABLE pasta ADD COLUMN write_key TEXT", params![]);

    // 旧三布尔模型映射到新模型。旧模型的五个隐私档位是四个布尔量的固定组合：
    //   public   : private=0, read_only=0, encrypt_server=0
    //   unlisted : private=1, read_only=0, encrypt_server=0
    //   readonly : private=1, read_only=1, encrypt_server=0
    //   private  : private=1, read_only=0, encrypt_server=1
    //   secret   : private=1, read_only=0, encrypt_server=1, encrypt_client=1
    // 旧代码里「写入需要密码」的判定条件是 `read_only || encrypt_server`，
    // 「读取需要密码」的判定条件是 `encrypt_server`，据此一一对应到新列。
    // 旧模型没有 refuse 这一档，因此只会映射出 allow(0) / password(1)。
    // 旧的 `encrypted_key` 就是 `encrypt(id, 密码)`，与新的校验值格式一致，
    // 因此可以直接搬给 read_key 与 write_key。
    let migrated = conn.execute(
        "UPDATE pasta SET
            listed = CASE WHEN private THEN 0 ELSE 1 END,
            read_access = CASE WHEN encrypt_server THEN 1 ELSE 0 END,
            write_access = CASE WHEN (read_only OR encrypt_server) THEN 1 ELSE 0 END,
            read_key = encrypted_key,
            write_key = encrypted_key",
        params![],
    );

    if migrated.is_ok() {
        log::warn!(
            "已将旧的隐私分档数据迁移到新的权限模型（read / write / list）。\
             旧的浏览器端加密上传无法再在浏览器里解密，其正文将保持密文。"
        );
    }
}

fn attachments_json(pasta: &Pasta) -> String {
    serde_json::to_string(&pasta.attachments).unwrap_or_default()
}

pub fn read_all() -> Vec<Pasta> {
    select_all_from_db()
}

pub fn update_all(pastas: &[Pasta]) {
    rewrite_all_to_db(pastas);
}

pub fn rewrite_all_to_db(pasta_data: &[Pasta]) {
    let conn = open();

    conn.execute("DROP TABLE IF EXISTS pasta;", params![])
        .expect("Failed to drop SQLite table for Pasta!");

    ensure_schema(&conn);

    for pasta in pasta_data.iter() {
        conn.execute(
            INSERT_SQL,
            params![
                pasta.id,
                pasta.content,
                pasta.file.as_ref().map_or("", |f| f.name.as_str()),
                pasta.file.as_ref().map_or(0, |f| f.size.as_u64()),
                pasta.extension,
                pasta.read_access.as_i32(),
                pasta.write_access.as_i32(),
                pasta.listed as i32,
                pasta.read_key.as_deref(),
                pasta.write_key.as_deref(),
                pasta.editable as i32,
                pasta.created,
                pasta.expiration,
                pasta.last_read,
                pasta.read_count,
                pasta.burn_after_reads,
                attachments_json(pasta),
                pasta.pasta_type,
            ],
        )
        .expect("Failed to insert pasta.");
    }
}

pub fn select_all_from_db() -> Vec<Pasta> {
    let conn = open();

    ensure_schema(&conn);

    let mut stmt = conn
        .prepare(&format!(
            "SELECT {SELECT_COLUMNS} FROM pasta ORDER BY created ASC"
        ))
        .expect("Failed to prepare SQL statement to load pastas");

    let pasta_iter = stmt
        .query_map([], |row| {
            Ok(Pasta {
                id: row.get(0)?,
                content: row.get(1)?,
                file: if let (Some(file_name), Some(file_size)) = (row.get(2)?, row.get(3)?) {
                    let file_size: u64 = file_size;
                    if file_name != "" && file_size != 0 {
                        Some(PastaFile {
                            name: file_name,
                            size: ByteSize::b(file_size),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                },
                extension: row.get(4)?,
                read_access: ReadAccess::from_i32(row.get(5)?),
                write_access: WriteAccess::from_i32(row.get(6)?),
                listed: row.get(7)?,
                read_key: row.get(8)?,
                write_key: row.get(9)?,
                editable: row.get(10)?,
                created: row.get(11)?,
                expiration: row.get(12)?,
                last_read: row.get(13)?,
                read_count: row.get(14)?,
                burn_after_reads: row.get(15)?,
                attachments: match row.get::<_, Option<String>>(16) {
                    Ok(Some(json)) => serde_json::from_str(&json).unwrap_or(None),
                    _ => None,
                },
                pasta_type: row.get(17)?,
            })
        })
        .expect("Failed to select Pastas from SQLite database.");

    pasta_iter
        .map(|r| r.expect("Failed to get pasta"))
        .collect::<Vec<Pasta>>()
}

pub fn insert(pasta: &Pasta) {
    let conn = open();

    ensure_schema(&conn);

    conn.execute(
        INSERT_SQL,
        params![
            pasta.id,
            pasta.content,
            pasta.file.as_ref().map_or("", |f| f.name.as_str()),
            pasta.file.as_ref().map_or(0, |f| f.size.as_u64()),
            pasta.extension,
            pasta.read_access.as_i32(),
            pasta.write_access.as_i32(),
            pasta.listed as i32,
            pasta.read_key.as_deref(),
            pasta.write_key.as_deref(),
            pasta.editable as i32,
            pasta.created,
            pasta.expiration,
            pasta.last_read,
            pasta.read_count,
            pasta.burn_after_reads,
            attachments_json(pasta),
            pasta.pasta_type,
        ],
    )
    .expect("Failed to insert pasta.");
}

pub fn update(pasta: &Pasta) {
    let conn = open();

    ensure_schema(&conn);

    conn.execute(
        UPDATE_SQL,
        params![
            pasta.id,
            pasta.content,
            pasta.file.as_ref().map_or("", |f| f.name.as_str()),
            pasta.file.as_ref().map_or(0, |f| f.size.as_u64()),
            pasta.extension,
            pasta.read_access.as_i32(),
            pasta.write_access.as_i32(),
            pasta.listed as i32,
            pasta.read_key.as_deref(),
            pasta.write_key.as_deref(),
            pasta.editable as i32,
            pasta.created,
            pasta.expiration,
            pasta.last_read,
            pasta.read_count,
            pasta.burn_after_reads,
            attachments_json(pasta),
            pasta.pasta_type,
        ],
    )
    .expect("Failed to update pasta.");
}

pub fn delete_by_id(id: u64) {
    let conn = open();

    conn.execute("DELETE FROM pasta WHERE id = ?1;", params![id])
        .expect("Failed to delete pasta.");
}
