use turso::{Builder, Connection};

pub struct TursoDB {
    conn: Connection,
}

impl TursoDB {
    /// Creates a new Catalog instance.
    /// Initializes the database connection and ensures the table exists.
    /// If the file does not exist, it will be created.
    pub async fn new(path: &str) -> turso::Result<Self> {
        let db = Builder::new_local(path).build().await?;
        let conn = db.connect()?;

        // Create the table if it doesn't exist
        // We use a simple key-value store for "information"
        conn.execute(
            r#"CREATE TABLE IF NOT EXISTS metadata (
                key TEXT PRIMARY KEY,
                value TEXT
            )"#,
            (),
        )
        .await?;

        Ok(Self { conn })
    }

    /// Stores a key-value pair in the catalog.
    /// If the key already exists, the value is updated.
    pub async fn set(&self, key: &str, value: &str) -> turso::Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO metadata (key, value) VALUES (?1, ?2)",
                [key, value],
            )
            .await?;
        Ok(())
    }

    /// Retrieves a value by its key.
    /// Returns None if the key does not exist.
    pub async fn get(&self, key: &str) -> turso::Result<Option<String>> {
        let mut rows = self
            .conn
            .query("SELECT value FROM metadata WHERE key = ?1", [key])
            .await?;

        match rows.next().await? {
            Some(row) => {
                let value: String = row.get(0)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
