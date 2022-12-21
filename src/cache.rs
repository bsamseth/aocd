use rusqlite::{Connection, Result};

pub struct Cache {
    year: u16,
    day: u8,
    session: String,
    connection: Connection,
}

impl Cache {
    pub fn new(year: u16, day: u8, session: &str) -> Self {
        let directory = std::env::var("AOC_CACHE_DIR")
            .or_else(|_| std::env::var("XDG_CACHE_HOME"))
            .unwrap_or_else(|_| shellexpand::tilde("~/.cache/aocd").to_string());

        std::fs::create_dir_all(&directory)
            .unwrap_or_else(|_| panic!("Faled to create cache directory: {}", directory));

        let connection = Connection::open(&format!("{}/aocd.sqlite", directory))
            .expect("Failed to open cache database");
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS puzzle_input (
                   session      TEXT NOT NULL,
                   year         INTEGER NOT NULL,
                   day          INTEGER NOT NULL,
                   input        TEXT NOT NULL,
                   PRIMARY KEY  (session, year, day)
                 )",
                [],
            )
            .expect("Failed to create puzzle_input cache table");
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS puzzle_answer (
                   session      TEXT NOT NULL,
                   year         INTEGER NOT NULL,
                   day          INTEGER NOT NULL,
                   part         INTEGER NOT NULL,
                   answer       TEXT NOT NULL,
                   correct      BOOLEAN NOT NULL,
                   response     TEXT NOT NULL,
                   PRIMARY KEY  (session, year, day, part, answer)
                  )",
                [],
            )
            .expect("Failed to create puzzle_answer cache table");

        Self {
            year,
            day,
            session: session.to_string(),
            connection,
        }
    }

    pub fn cache_answer_response(
        &self,
        part: u8,
        answer: impl ToString,
        response: &str,
        correct: bool,
    ) {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO puzzle_answer (session, year, day, part, answer, correct, response)
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
                (
                    &self.session,
                    self.year,
                    self.day,
                    part,
                    answer.to_string(),
                    correct,
                    response,
                ),
            )
            .expect("Failed to cache puzzle answer response");
    }

    pub fn get_correct_answer(&self, part: u8) -> Result<String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT answer 
                 FROM puzzle_answer 
                 WHERE session = ? AND year = ? AND day = ? AND part = ? AND correct",
            )
            .expect("Failed to prepare get correct answer statement");
        let mut rows = statement.query((&self.session, self.year, self.day, part))?;
        match rows.next()? {
            Some(row) => Ok(row.get(0)?),
            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    pub fn get_answer_response(&self, part: u8, answer: &str) -> Result<String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT response
                 FROM puzzle_answer
                 WHERE session = ? AND year = ? AND day = ? AND part = ? AND answer = ?",
            )
            .expect("Failed to prepare get_answer_response query");
        let mut rows =
            statement.query((&self.session, self.year, self.day, part, answer.to_string()))?;
        match rows.next()? {
            Some(cached) => Ok(cached.get(0)?),
            _ => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    pub fn get_input(&self) -> Result<String> {
        let mut statement = self
            .connection
            .prepare("SELECT input FROM puzzle_input WHERE session = ? AND year = ? AND day = ?")
            .expect("Failed to prepare puzzle_input query");
        let row = statement
            .query_map((&self.session, self.year, self.day), |row| row.get(0))?
            .next();
        match row {
            Some(input) => input,
            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    pub fn cache_input(&self, input: &str) {
        self.connection
            .execute(
                "INSERT OR REPLACE INTO puzzle_input (session, year, day, input) VALUES (?, ?, ?, ?)",
                (&self.session, self.year, self.day, input),
            )
            .expect("Failed to insert puzzle_input into cache");
    }
}
