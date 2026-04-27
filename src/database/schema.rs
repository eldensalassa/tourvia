/// SQL DDL statements for initializing the Tourvia database schema.

pub const CREATE_TOURNAMENTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS tournaments (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    tournament_type TEXT NOT NULL DEFAULT 'Single Elimination',
    participant_count INTEGER NOT NULL DEFAULT 0,
    status          TEXT NOT NULL DEFAULT 'Draft',
    created_at      TEXT NOT NULL,
    description     TEXT NOT NULL DEFAULT '',
    game_name       TEXT NOT NULL DEFAULT ''
);
";

pub const CREATE_PARTICIPANTS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS participants (
    id              TEXT PRIMARY KEY,
    tournament_id   TEXT NOT NULL,
    name            TEXT NOT NULL,
    seed            INTEGER NOT NULL DEFAULT 0,
    logo_data       BLOB,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE
);
";

pub const CREATE_ROUNDS_TABLE: &str = "
CREATE TABLE IF NOT EXISTS rounds (
    id              TEXT PRIMARY KEY,
    tournament_id   TEXT NOT NULL,
    round_number    INTEGER NOT NULL,
    name            TEXT NOT NULL,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE
);
";

pub const CREATE_MATCHES_TABLE: &str = "
CREATE TABLE IF NOT EXISTS matches (
    id              TEXT PRIMARY KEY,
    tournament_id   TEXT NOT NULL,
    round_id        TEXT NOT NULL,
    match_order     INTEGER NOT NULL,
    player1_id      TEXT,
    player2_id      TEXT,
    player1_name    TEXT NOT NULL DEFAULT '',
    player2_name    TEXT NOT NULL DEFAULT '',
    score1          INTEGER NOT NULL DEFAULT 0,
    score2          INTEGER NOT NULL DEFAULT 0,
    winner_id       TEXT,
    status          TEXT NOT NULL DEFAULT 'Pending',
    next_match_id   TEXT,
    next_match_slot INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (tournament_id) REFERENCES tournaments(id) ON DELETE CASCADE,
    FOREIGN KEY (round_id) REFERENCES rounds(id) ON DELETE CASCADE
);
";

/// Migration statements for existing databases (add new columns if missing).
pub const MIGRATIONS: &[&str] = &[
    "ALTER TABLE tournaments ADD COLUMN description TEXT NOT NULL DEFAULT '';",
    "ALTER TABLE tournaments ADD COLUMN game_name TEXT NOT NULL DEFAULT '';",
    "ALTER TABLE participants ADD COLUMN logo_data BLOB;",
];
