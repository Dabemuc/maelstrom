use std::path::Path;
use turso::{Builder, Connection};

pub struct TursoDB {
    conn: Connection,
}

#[derive(Debug, Clone)]
pub struct ImageDO {
    pub path: String,
    pub hash: String,
}

impl TursoDB {
    /// Opens an existing catalog. Fails if the file does not exist.
    pub async fn open(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        if !Path::new(path).exists() {
            return Err(format!("Catalog file not found: {}", path).into());
        }

        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;

        Ok(Self { conn })
    }

    /// Creates a new catalog (or opens existing) and initializes the schema.
    pub async fn create(path: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS imported_paths (
                path TEXT PRIMARY KEY
            )"#,
            (),
        )
        .await?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT
            )"#,
            (),
        )
        .await?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS images (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    content_hash TEXT NOT NULL UNIQUE,
                    path TEXT NOT NULL
                )"#,
            (),
        )
        .await?;

        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS edit_graphs (
                content_hash TEXT PRIMARY KEY,
                graph_json TEXT NOT NULL,
                updated_at TEXT
            )"#,
            (),
        )
        .await?;

        Ok(Self { conn })
    }

    /// Inserts an image (ignored if already exists).
    pub async fn add_image(&self, content_hash: &str, path: &str) -> turso::Result<ImageDO> {
        let mut rows = self
            .conn
            .query(
                r#"
                INSERT INTO images (content_hash, path)
                VALUES (?1, ?2)
                ON CONFLICT(content_hash)
                DO UPDATE SET path = images.path
                RETURNING path, content_hash
                "#,
                [content_hash, path],
            )
            .await?;

        let row = rows
            .next()
            .await?
            .expect("RETURNING must return exactly one row");

        Ok(ImageDO {
            path: row.get(0)?,
            hash: row.get(1)?,
        })
    }

    /// Checks whether an image with this hash already exists.
    pub async fn image_exists(&self, content_hash: &str) -> turso::Result<bool> {
        let mut rows = self
            .conn
            .query(
                "SELECT 1 FROM images WHERE content_hash = ?1 LIMIT 1",
                [content_hash],
            )
            .await?;

        Ok(rows.next().await?.is_some())
    }

    pub async fn get_edit_graph_json(
        &self,
        content_hash: &str,
    ) -> turso::Result<Option<String>> {
        let mut rows = self
            .conn
            .query(
                "SELECT graph_json FROM edit_graphs WHERE content_hash = ?1 LIMIT 1",
                [content_hash],
            )
            .await?;

        if let Some(row) = rows.next().await? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_edit_graph_json(
        &self,
        content_hash: &str,
        graph_json: &str,
    ) -> turso::Result<()> {
        self.conn
            .execute(
                r#"
                INSERT INTO edit_graphs (content_hash, graph_json, updated_at)
                VALUES (?1, ?2, datetime('now'))
                ON CONFLICT(content_hash)
                DO UPDATE SET graph_json = excluded.graph_json, updated_at = datetime('now')
                "#,
                [content_hash, graph_json],
            )
            .await?;
        Ok(())
    }

    pub async fn ensure_edit_graph_json(
        &self,
        content_hash: &str,
        graph_json: &str,
    ) -> turso::Result<()> {
        self.conn
            .execute(
                r#"
                INSERT OR IGNORE INTO edit_graphs (content_hash, graph_json, updated_at)
                VALUES (?1, ?2, datetime('now'))
                "#,
                [content_hash, graph_json],
            )
            .await?;
        Ok(())
    }

    /// Returns the hashes of all images for a given path (and subpaths)
    pub async fn get_image_dos_by_path(&self, path: &str) -> turso::Result<Vec<ImageDO>> {
        let subpath_pattern = format!("{}/%", path); // matches all subpaths
        let mut rows = self
            .conn
            .query(
                "SELECT path, content_hash FROM images WHERE path = ?1 OR path LIKE ?2",
                [path, &subpath_pattern],
            )
            .await?;

        let mut image_dos = Vec::new();
        while let Some(row) = rows.next().await? {
            let path: String = row.get(0)?;
            let hash: String = row.get(1)?;
            image_dos.push(ImageDO {
                path: path,
                hash: hash,
            });
        }
        Ok(image_dos)
    }

    /// Adds a directory path to the imported paths list.
    pub async fn add_imported_path(&self, path: &str) -> turso::Result<()> {
        self.conn
            .execute(
                "INSERT OR IGNORE INTO imported_paths (path) VALUES (?1)",
                [path],
            )
            .await?;
        Ok(())
    }

    /// Retrieves all imported directory paths.
    pub async fn get_imported_paths(&self) -> turso::Result<Vec<String>> {
        let mut rows = self
            .conn
            .query("SELECT path FROM imported_paths", ())
            .await?;

        let mut paths = Vec::new();
        while let Some(row) = rows.next().await? {
            let path: String = row.get(0)?;
            paths.push(path);
        }
        Ok(paths)
    }

    /// Sets the catalog version in the meta table.
    pub async fn set_version(&self, version: u16) -> turso::Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('version', ?1)",
                [version.to_string()],
            )
            .await?;
        Ok(())
    }

    /// Retrieves the catalog version from the meta table.
    pub async fn get_version(
        &self,
    ) -> Result<Option<u16>, Box<dyn std::error::Error + Send + Sync>> {
        let mut rows = self
            .conn
            .query("SELECT value FROM meta WHERE key = 'version'", ())
            .await?;

        if let Some(row) = rows.next().await? {
            let version_str: String = row.get(0)?;
            let version = version_str.parse::<u16>()?;
            Ok(Some(version))
        } else {
            Ok(None)
        }
    }
}
