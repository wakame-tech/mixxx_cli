use super::repo::{AsRepo, Repo};
use anyhow::Result;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Debug, Clone, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum CueType {
    Invalid,
    HotCue,
    MainCue,
    Beat,
    Loop,
    Jump,
    Intro,
    Outro,
    N60dBSound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cue {
    pub id: i32,
    pub track_id: i32,
    pub r#type: CueType,
    pub position: f32,
    pub length: f32,
    pub hotcue: u8,
}

impl<'a> AsRepo<'a> for Cue {
    fn repo(conn: &'a rusqlite::Connection) -> super::repo::Repo<'a, Self> {
        Repo::new(conn, "cues")
    }
}

impl<'a> Repo<'a, Cue> {
    pub fn hot_cues_by_track_id(&self, track_id: i32) -> Result<Vec<Cue>> {
        let mut stmt = self.conn.prepare(
            format!("SELECT * FROM {} WHERE track_id=?1 AND type=?2", self.table).as_str(),
        )?;
        self.query(&mut stmt, params![track_id, CueType::HotCue as u8])
    }

    pub fn hot_cue_by_track_id(&self, track_id: i32, hotcue: u8) -> Result<Cue> {
        let mut stmt = self.conn.prepare(
            format!(
                "SELECT * FROM {} WHERE track_id=?1 AND type=?2 AND hotcue=?3",
                self.table
            )
            .as_str(),
        )?;
        self.query(&mut stmt, params![track_id, CueType::HotCue as u8, hotcue])
            .and_then(|res| res.get(0).cloned().ok_or(anyhow::anyhow!("not found")))
    }
}
