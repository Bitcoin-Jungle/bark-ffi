
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::Context;
use log::{info, warn};
use rusqlite::{OpenFlags, OptionalExtension};

use bark::persist::sqlite::{SqliteClient, DEFAULT_DB_FILE};

use crate::error::Error;
use crate::types::Network;

/// Database filename bark-ffi used before v0.3.0. Still honoured on open.
const LEGACY_DB_FILE: &str = "bark.sqlite";

/// What [`read_datadir_info`] found in a wallet directory.
#[derive(Clone, Debug, uniffi::Record)]
pub struct DatadirInfo {
    /// Path of the database file that was read.
    pub db_file: String,
    pub network: Network,
    /// Fingerprint of the wallet seed, as `Wallet::fingerprint` returns it.
    /// Compare it against the seed you are about to open with to catch a
    /// restore-over-the-wrong-wallet before any file is touched.
    pub fingerprint: String,
}

/// The database file an open of `datadir` would use, if one exists.
fn existing_db_path(datadir: &Path) -> Option<PathBuf> {
    let db_path = datadir.join(DEFAULT_DB_FILE);
    if db_path.is_file() {
        return Some(db_path);
    }
    let compat = datadir.join(LEGACY_DB_FILE);
    if compat.is_file() {
        return Some(compat);
    }
    None
}

/// Whether `datadir` holds an initialized bark wallet.
///
/// Answers without side effects: the database is opened read-only, no
/// migration runs and no file or directory is created. Returns `false` for a
/// missing or empty directory and for a database that never got its wallet
/// properties (for example after a crash during creation). Returns an error
/// when a database exists but cannot be read, so a corrupt wallet is never
/// mistaken for an absent one.
#[uniffi::export]
pub fn wallet_exists(datadir: String) -> Result<bool, Error> {
    Ok(read_datadir_info(datadir)?.is_some())
}

/// Network and seed fingerprint of the wallet in `datadir`, or `None` when no
/// wallet is initialized there. Same side-effect guarantees as
/// [`wallet_exists`].
///
/// Not named `datadir_info`: the Go bindings would export it as `DatadirInfo`,
/// which collides with the record of the same name.
#[uniffi::export]
pub fn read_datadir_info(datadir: String) -> Result<Option<DatadirInfo>, Error> {
    let Some(db_path) = existing_db_path(Path::new(&datadir)) else {
        return Ok(None);
    };
    Ok(read_wallet_properties(&db_path)?)
}

fn read_wallet_properties(db_path: &Path) -> anyhow::Result<Option<DatadirInfo>> {
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ).with_context(|| format!("failed to open {} read-only", db_path.display()))?;

    // A database can exist without the properties table: `create` writes the
    // schema before the properties row, and a crash in between leaves exactly
    // that. Treat it as "no wallet" rather than as an error.
    let has_table: i64 = conn.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = 'bark_properties'",
        [],
        |row| row.get(0),
    ).context("failed to inspect database schema")?;
    if has_table == 0 {
        return Ok(None);
    }

    let row = conn.query_row(
        "SELECT network, fingerprint FROM bark_properties LIMIT 1",
        [],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    ).optional().context("failed to read wallet properties")?;

    let Some((network, fingerprint)) = row else {
        return Ok(None);
    };
    let network = bitcoin::Network::from_str(&network)
        .with_context(|| format!("wallet properties hold an unknown network {:?}", network))?;

    Ok(Some(DatadirInfo {
        db_file: db_path.to_string_lossy().into_owned(),
        network: network.into(),
        fingerprint,
    }))
}


/// Open a SQLite database connection.
///
/// # Arguments
///
/// * `datadir` - The directory containing the wallet data
///
/// # Returns
///
/// An `Arc<SqliteClient>` that can be shared across multiple wallet instances.
pub fn open_sqlite_db(datadir: &str) -> anyhow::Result<Arc<SqliteClient>> {
    let datadir_path = PathBuf::from(datadir);

    // Ensure the directory exists
    std::fs::create_dir_all(&datadir_path)
        .with_context(|| format!("Failed to create datadir {}", datadir_path.display(),))?;

    let db_path = {
        let db_path = datadir_path.join(DEFAULT_DB_FILE);

        // bark-ffi pre v0.3.0 used bark.sqlite as the db filename,
        let compat_db_path = datadir_path.join(LEGACY_DB_FILE);

        if !db_path.exists() && compat_db_path.exists() {
            warn!("You're using the old default filename bark.sqlite for your SQLite DB, \
                while the new default is db.sqlite. You might want to rename your file.");
            compat_db_path
        } else {
            db_path
        }
    };

    // Open new connection
    info!("[DB] Opening new connection for {}", db_path.display());
    let db = Arc::new(
        SqliteClient::open(&db_path)
            .with_context(|| format!("Failed to open database at {}", db_path.display(),))?,
    );

    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_datadir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "bark-ffi-datadir-{}-{}", name, std::process::id(),
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_properties(db: &Path, network: &str, fingerprint: &str) {
        let conn = rusqlite::Connection::open(db).unwrap();
        conn.execute_batch(
            "CREATE TABLE bark_properties (
                id TEXT PRIMARY KEY,
                network TEXT NOT NULL,
                fingerprint TEXT NOT NULL
            );",
        ).unwrap();
        conn.execute(
            "INSERT INTO bark_properties (id, network, fingerprint) VALUES ('wallet', ?1, ?2)",
            [network, fingerprint],
        ).unwrap();
    }

    #[test]
    fn missing_datadir_is_not_a_wallet() {
        let dir = tmp_datadir("missing");
        let nested = dir.join("does-not-exist");
        assert!(!wallet_exists(nested.to_string_lossy().into_owned()).unwrap());
        // Probing must not create anything.
        assert!(!nested.exists());
    }

    #[test]
    fn empty_datadir_is_not_a_wallet() {
        let dir = tmp_datadir("empty");
        assert!(!wallet_exists(dir.to_string_lossy().into_owned()).unwrap());
        assert!(std::fs::read_dir(&dir).unwrap().next().is_none(), "probe created a file");
    }

    #[test]
    fn database_without_properties_is_not_a_wallet() {
        let dir = tmp_datadir("no-props");
        rusqlite::Connection::open(dir.join(DEFAULT_DB_FILE)).unwrap()
            .execute_batch("CREATE TABLE unrelated (x INTEGER);").unwrap();
        assert!(!wallet_exists(dir.to_string_lossy().into_owned()).unwrap());
    }

    #[test]
    fn initialized_wallet_reports_network_and_fingerprint() {
        let dir = tmp_datadir("init");
        write_properties(&dir.join(DEFAULT_DB_FILE), "signet", "deadbeef");
        let info = read_datadir_info(dir.to_string_lossy().into_owned()).unwrap().unwrap();
        assert!(matches!(info.network, Network::Signet));
        assert_eq!(info.fingerprint, "deadbeef");
        assert!(info.db_file.ends_with(DEFAULT_DB_FILE));
        assert!(wallet_exists(dir.to_string_lossy().into_owned()).unwrap());
    }

    #[test]
    fn legacy_filename_is_honoured() {
        let dir = tmp_datadir("legacy");
        write_properties(&dir.join(LEGACY_DB_FILE), "bitcoin", "cafebabe");
        let info = read_datadir_info(dir.to_string_lossy().into_owned()).unwrap().unwrap();
        assert!(matches!(info.network, Network::Bitcoin));
        assert!(info.db_file.ends_with(LEGACY_DB_FILE));
    }

    #[test]
    fn unreadable_database_is_an_error_not_absent() {
        let dir = tmp_datadir("corrupt");
        std::fs::write(dir.join(DEFAULT_DB_FILE), b"this is not a sqlite file").unwrap();
        assert!(wallet_exists(dir.to_string_lossy().into_owned()).is_err());
    }
}
