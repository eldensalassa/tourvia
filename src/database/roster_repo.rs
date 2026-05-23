use rusqlite::params;
use crate::domain::repositories::RosterRepository;
use crate::domain::roster::Roster;
use super::connection::Database;

impl RosterRepository for Database {
    fn create_roster(&self, roster: &Roster) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO rosters (id, name, game, logo_data) VALUES (?1, ?2, ?3, ?4)",
            params![roster.id, roster.name, roster.game, roster.logo_data],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_rosters(&self) -> Result<Vec<Roster>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, name, game, logo_data FROM rosters ORDER BY name ASC")
            .map_err(|e| e.to_string())?;

        let roster_iter = stmt
            .query_map([], |row| {
                Ok(Roster {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    game: row.get(2)?,
                    logo_data: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut rosters = Vec::new();
        for r in roster_iter {
            rosters.push(r.map_err(|e| e.to_string())?);
        }
        Ok(rosters)
    }

    fn delete_roster(&self, id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM rosters WHERE id = ?1", params![id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update_roster(&self, roster: &Roster) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE rosters SET name = ?1, game = ?2, logo_data = ?3 WHERE id = ?4",
            params![roster.name, roster.game, roster.logo_data, roster.id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn add_roster_member(&self, member: &crate::domain::roster::RosterMember) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO roster_members (id, roster_id, name, profile_picture) VALUES (?1, ?2, ?3, ?4)",
            params![member.id, member.roster_id, member.name, member.profile_picture],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_roster_members(&self, roster_id: &str) -> Result<Vec<crate::domain::roster::RosterMember>, String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, roster_id, name, profile_picture FROM roster_members WHERE roster_id = ?1 ORDER BY name ASC")
            .map_err(|e| e.to_string())?;

        let member_iter = stmt
            .query_map(params![roster_id], |row| {
                Ok(crate::domain::roster::RosterMember {
                    id: row.get(0)?,
                    roster_id: row.get(1)?,
                    name: row.get(2)?,
                    profile_picture: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;

        let mut members = Vec::new();
        for m in member_iter {
            members.push(m.map_err(|e| e.to_string())?);
        }
        Ok(members)
    }

    fn delete_roster_member(&self, member_id: &str) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM roster_members WHERE id = ?1", params![member_id])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn update_roster_member(&self, member: &crate::domain::roster::RosterMember) -> Result<(), String> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE roster_members SET name = ?1, profile_picture = ?2 WHERE id = ?3",
            params![member.name, member.profile_picture, member.id],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }
}
