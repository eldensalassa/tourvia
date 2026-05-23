use rusqlite::params;
use crate::domain::game::Game;
use crate::domain::repositories::GameRepository;
use super::connection::Database;

impl GameRepository for Database {
    fn create_game(&self, game: &Game) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO games (id, name) VALUES (?1, ?2)",
            params![game.id, game.name],
        )
        .map_err(|e| format!("Failed to create game: {}", e))?;
        Ok(())
    }

    fn get_games(&self) -> Result<Vec<Game>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, name FROM games ORDER BY name ASC").map_err(|e| e.to_string())?;
        
        let game_iter = stmt.query_map([], |row| {
            Ok(Game {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut games = Vec::new();
        for game in game_iter {
            games.push(game.map_err(|e| e.to_string())?);
        }
        Ok(games)
    }

    fn delete_game(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM games WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete game: {}", e))?;
        Ok(())
    }
}
