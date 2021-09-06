use super::Error;
use r2d2::Pool;
use r2d2_sqlite::rusqlite::{Connection, OptionalExtension};
use r2d2_sqlite::SqliteConnectionManager;
use serenity::prelude::{RwLock, TypeMapKey};
use std::sync::Arc;

pub struct QuoteDatabase {
    db: Pool<SqliteConnectionManager>,
}

pub struct QuoteResult {
    pub user_id: u64,
    pub quote: String,
}

impl TypeMapKey for QuoteDatabase {
    type Value = Arc<QuoteDatabase>;
}

impl QuoteDatabase {
    pub fn open() -> Result<Self, Error> {
        Ok(QuoteDatabase {
            db: Pool::new(SqliteConnectionManager::file("quotes.db"))?,
        })
    }

    pub fn new() -> Result<(), Error> {
        let db = Connection::open("quotes.db")?;

        db.execute("CREATE TABLE users (user_id INTEGER PRIMARY KEY)", [])?;
        db.execute("CREATE TABLE quotes (quote_id INTEGER PRIMARY KEY, user_id INTEGER NOT NULL, quote TEXT NOT NULL, FOREIGN KEY (user_id) REFERENCES users (user_id))", [])?;

        Ok(())
    }

    pub fn add_quote(&self, user_id: u64, quote: String) -> Result<(), Error> {
        let result: Option<u64> = self
            .db
            .get()?
            .query_row(
                "SELECT user_id FROM users WHERE user_id=:id",
                &[(":id", &user_id.to_string())],
                |r| r.get(0),
            )
            .optional()?;

        if result.is_none() {
            self.db.get()?.execute(
                "INSERT INTO users (user_id) VALUES (:id)",
                &[(":id", &user_id.to_string())],
            )?;
        }

        self.db.get()?.execute(
            "INSERT INTO quotes (user_id, quote) VALUES (:id, :quote)",
            &[(":id", &user_id.to_string()), (":quote", &quote)],
        )?;

        Ok(())
    }

    pub fn get_quotes(&self, quote_fragment: String) -> Result<Vec<QuoteResult>, Error> {
        let conn = self.db.get()?;

        let mut statement =
            conn.prepare("SELECT user_id, quote FROM quotes WHERE like(:fragment, quote)")?;

        let rows = statement.query_map(&[(":fragment", &format!("%{}%", quote_fragment))], |r| {
            Ok(QuoteResult {
                user_id: r.get(0)?,
                quote: r.get(1)?,
            })
        })?;

        Ok(rows.map(|r| r.unwrap()).collect::<Vec<QuoteResult>>())
    }
}
