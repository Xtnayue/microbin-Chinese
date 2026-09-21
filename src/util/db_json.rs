use serde_json::json;
use std::fs::File;
use std::io;
use std::io::{BufReader, BufWriter};

use crate::{Pasta, ARGS};

/// JSON 数据库文件的路径。
///
/// 与 SQLite 后端（`{data_dir}/database.sqlite`）保持一致，跟随 `--data-dir`。
/// 上游此前把这里硬编码成 `pasta_data/database.json`，导致 `--json-db` 下
/// `--data-dir` 完全不生效；且一旦 `pasta_data/` 不存在，启动阶段就会
/// 在创建临时文件时 panic。
fn database_path() -> String {
    format!("{}/database.json", ARGS.data_dir)
}

pub fn read_all() -> Vec<Pasta> {
    load_from_file().expect("Failed to load pastas from JSON")
}

pub fn update_all(pastas: &Vec<Pasta>) {
    save_to_file(pastas);
}

fn save_to_file(pasta_data: &Vec<Pasta>) {
    // This uses a two stage write. First we write to a new file, if this fails
    // only the new pasta's are lost. Then we replace the current database with
    // the new file. This either succeeds or fails. The database is never left
    // in an undefined state.
    let database_path = database_path();
    let tmp_file_path = database_path.clone() + ".tmp";
    let tmp_file = File::create(&tmp_file_path).expect(&format!(
        "failed to create temporary database file for writing. path: {tmp_file_path}"
    ));

    let writer = BufWriter::new(tmp_file);
    serde_json::to_writer(writer, &pasta_data)
        .expect("Should be able to write out data to database file");
    std::fs::rename(tmp_file_path, database_path).expect("Could not update database");
}

/// 把旧的「隐私分档」记录就地映射到新的权限列。
///
/// 旧 JSON 记录里没有 `read_access` / `write_access` / `listed` 三个字段，
/// 若直接按 `#[serde(default)]` 反序列化，会全部落到最保守的一侧
/// （read=password、write=refuse、不列出），使旧数据凭空变得不可读也不可写。
/// 因此这里在反序列化前按旧字段把三项权限补齐。
///
/// 映射规则与 SQLite 的迁移完全一致（旧代码里「写入需要密码」的判定是
/// `readonly || encrypt_server`，不是单看 `readonly`）：
///
/// ```text
/// listed       = !private
/// read_access  = encrypt_server               ? "password" : "allow"
/// write_access = readonly || encrypt_server   ? "password" : "allow"
/// read_key / write_key = encrypted_key
/// ```
///
/// 旧的 `encrypted_key` 就是 `encrypt(上传 id, 密码)`，与新的校验值格式相同，
/// 因此两个 key 都可以直接沿用。返回 `true` 表示该条记录被迁移过。
fn migrate_legacy_permissions(value: &mut serde_json::Value) -> bool {
    let Some(object) = value.as_object_mut() else {
        return false;
    };

    // 已经有新字段，说明是迁移过的记录或新建的记录
    if object.contains_key("read_access") {
        return false;
    }

    let flag = |key: &str| {
        object
            .get(key)
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
    };

    let private = flag("private");
    let readonly = flag("readonly");
    let encrypt_server = flag("encrypt_server");

    let encrypted_key = object
        .get("encrypted_key")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    object.insert(
        String::from("read_access"),
        json!(if encrypt_server { "password" } else { "allow" }),
    );
    object.insert(
        String::from("write_access"),
        json!(if readonly || encrypt_server {
            "password"
        } else {
            "allow"
        }),
    );
    object.insert(String::from("listed"), json!(!private));
    object.insert(String::from("read_key"), encrypted_key.clone());
    object.insert(String::from("write_key"), encrypted_key);

    true
}

fn load_from_file() -> io::Result<Vec<Pasta>> {
    let database_path = database_path();
    let file = File::open(&database_path);
    match file {
        Ok(_) => {
            let reader = BufReader::new(file.unwrap());

            // 先按通用 JSON 读出来，才能识别并迁移旧的权限字段
            let mut values: Vec<serde_json::Value> = match serde_json::from_reader(reader) {
                Ok(values) => values,
                _ => Vec::new(),
            };

            let mut migrated = 0usize;
            for value in values.iter_mut() {
                if migrate_legacy_permissions(value) {
                    migrated += 1;
                }
            }

            if migrated > 0 {
                log::warn!(
                    "已将 {migrated} 条旧的隐私分档记录迁移到新的权限模型（read / write / list）。\
                     旧的浏览器端加密上传无法再在浏览器里解密，其正文将保持密文。"
                );
            }

            let mut pastas = Vec::with_capacity(values.len());
            for value in values {
                match serde_json::from_value::<Pasta>(value) {
                    Ok(pasta) => pastas.push(pasta),
                    Err(error) => log::error!("Skipped an unreadable record in the JSON database: {error}"),
                }
            }

            Ok(pastas)
        }
        Err(_) => {
            log::info!("Database file {database_path} not found!");
            save_to_file(&Vec::<Pasta>::new());

            log::info!("Database file {database_path} created.");
            load_from_file()
        }
    }
}
