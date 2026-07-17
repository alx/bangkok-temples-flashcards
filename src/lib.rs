use uniffi::Object;
use rusqlite::{params, Connection, Result as SqliteResult};
use std::sync::Mutex;

#[derive(Clone, Debug)]
pub struct Temple {
    pub id: u32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub image_path: String,
    pub description: String,
}

#[derive(Clone, Debug)]
pub struct QuizCard {
    pub temple_id: u32,
    pub temple_name: String,
    pub image_path: String,
    pub latitude: f64,
    pub longitude: f64,
    pub quiz_type: String, // "image" or "location"
}

#[derive(Clone, Debug)]
pub struct QuizResult {
    pub is_correct: bool,
    pub score: u32,
    pub streak: u32,
}

#[object]
pub struct TempleQuizEngine {
    db: Mutex<Connection>,
}

#[uniffi::export]
impl TempleQuizEngine {
    #[uniffi::constructor]
    pub fn new() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("Database error: {}", e))?;

        // Initialize schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS temples (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                latitude REAL NOT NULL,
                longitude REAL NOT NULL,
                image_path TEXT NOT NULL,
                description TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS quiz_progress (
                id INTEGER PRIMARY KEY,
                temple_id INTEGER NOT NULL,
                correct_count INTEGER DEFAULT 0,
                wrong_count INTEGER DEFAULT 0,
                streak INTEGER DEFAULT 0
            );"
        ).map_err(|e| format!("Schema error: {}", e))?;

        Ok(TempleQuizEngine {
            db: Mutex::new(conn),
        })
    }

    pub fn add_temple(
        &self,
        id: u32,
        name: String,
        latitude: f64,
        longitude: f64,
        image_path: String,
        description: String,
    ) -> Result<(), String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        db.execute(
            "INSERT INTO temples (id, name, latitude, longitude, image_path, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![id, &name, latitude, longitude, &image_path, &description],
        ).map_err(|e| format!("Insert error: {}", e))?;
        Ok(())
    }

    pub fn get_all_temples(&self) -> Result<Vec<Temple>, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare("SELECT id, name, latitude, longitude, image_path, description FROM temples")
            .map_err(|e| e.to_string())?;

        let temples = stmt.query_map([], |row| {
            Ok(Temple {
                id: row.get(0)?,
                name: row.get(1)?,
                latitude: row.get(2)?,
                longitude: row.get(3)?,
                image_path: row.get(4)?,
                description: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())?;

        Ok(temples)
    }

    pub fn get_random_quiz_card(&self, quiz_type: String) -> Result<QuizCard, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare(
            "SELECT id, name, latitude, longitude, image_path FROM temples ORDER BY RANDOM() LIMIT 1"
        ).map_err(|e| e.to_string())?;

        let card = stmt.query_row([], |row| {
            Ok(QuizCard {
                temple_id: row.get(0)?,
                temple_name: row.get(1)?,
                latitude: row.get(2)?,
                longitude: row.get(3)?,
                image_path: row.get(4)?,
                quiz_type,
            })
        }).map_err(|e| e.to_string())?;

        Ok(card)
    }

    pub fn check_answer(&self, temple_id: u32, user_answer: String, correct_name: String) -> Result<QuizResult, String> {
        let is_correct = user_answer.to_lowercase().trim() == correct_name.to_lowercase().trim();

        let db = self.db.lock().map_err(|e| e.to_string())?;

        // Update progress
        let result = if is_correct {
            db.execute(
                "INSERT INTO quiz_progress (temple_id, correct_count, streak)
                 VALUES (?1, 1, 1)
                 ON CONFLICT(temple_id) DO UPDATE SET
                   correct_count = correct_count + 1,
                   streak = streak + 1",
                params![temple_id],
            ).map_err(|e| e.to_string())?;

            QuizResult {
                is_correct: true,
                score: 10,
                streak: 1,
            }
        } else {
            db.execute(
                "INSERT INTO quiz_progress (temple_id, wrong_count, streak)
                 VALUES (?1, 1, 0)
                 ON CONFLICT(temple_id) DO UPDATE SET
                   wrong_count = wrong_count + 1,
                   streak = 0",
                params![temple_id],
            ).map_err(|e| e.to_string())?;

            QuizResult {
                is_correct: false,
                score: 0,
                streak: 0,
            }
        };

        Ok(result)
    }

    pub fn get_stats(&self) -> Result<String, String> {
        let db = self.db.lock().map_err(|e| e.to_string())?;
        let mut stmt = db.prepare(
            "SELECT COUNT(*) as total, SUM(correct_count) as correct_count FROM quiz_progress"
        ).map_err(|e| e.to_string())?;

        let (total, correct) = stmt.query_row([], |row| {
            Ok((row.get::<_, i32>(0)?, row.get::<_, Option<i32>>(1)?))
        }).map_err(|e| e.to_string())?;

        let correct_count = correct.unwrap_or(0);
        let accuracy = if total > 0 {
            (correct_count as f64 / total as f64 * 100.0) as u32
        } else {
            0
        };

        Ok(format!("Total: {}, Correct: {}, Accuracy: {}%", total, correct_count, accuracy))
    }
}

uniffi::setup_scaffolding!();
