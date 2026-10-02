use crate::domain::sentence::{CardStatus, Sentence};
use crate::error::{AppError, Result};
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub struct AnkiPaths {
    pub collection_db: PathBuf,
    pub media_dir: PathBuf,
}

pub struct AnkiScanner;

impl AnkiScanner {
    pub fn locate_default_paths() -> Result<AnkiPaths> {
        let home = std::env::var("HOME").map_err(|e| AppError::Audio(e.to_string()))?;
        let anki2_base = PathBuf::from(home).join(".local/share/Anki2");

        if !anki2_base.exists() {
            return Err(AppError::CollectionNotFound(anki2_base));
        }

        let entries = fs::read_dir(&anki2_base)?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let db = path.join("collection.anki2");
                let media = path.join("collection.media");
                if db.exists() && media.exists() {
                    return Ok(AnkiPaths {
                        collection_db: db,
                        media_dir: media,
                    });
                }
            }
        }

        Err(AppError::CollectionNotFound(anki2_base))
    }

    pub fn open_read_only(db_path: &Path) -> Result<Connection> {
        let conn = Connection::open_with_flags(db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        conn.create_collation("unicase", |a, b| a.to_lowercase().cmp(&b.to_lowercase()))?;
        Ok(conn)
    }

    pub fn load_fields_map(conn: &Connection) -> Result<HashMap<i64, HashMap<usize, String>>> {
        let mut map: HashMap<i64, HashMap<usize, String>> = HashMap::new();
        let mut stmt = conn.prepare("SELECT ntid, ord, name FROM fields")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, usize>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;

        for item in rows {
            let (ntid, ord, name) = item?;
            map.entry(ntid).or_default().insert(ord, name);
        }
        Ok(map)
    }

    pub fn load_latest_reviews(conn: &Connection) -> Result<HashMap<i64, i64>> {
        let mut rev_map = HashMap::new();
        let mut stmt = conn.prepare("SELECT cid, max(id) FROM revlog GROUP BY cid")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))?;

        for item in rows {
            let (cid, last_ts) = item?;
            rev_map.insert(cid, last_ts);
        }
        Ok(rev_map)
    }

    pub fn scan_reviewed_sentences(paths: &AnkiPaths) -> Result<Vec<Sentence>> {
        let conn = Self::open_read_only(&paths.collection_db)?;
        let fields_map = Self::load_fields_map(&conn)?;
        let rev_map = Self::load_latest_reviews(&conn)?;

        let query = "
            SELECT c.id, c.nid, c.ivl, c.reps, c.type, c.queue, n.mid, n.flds, n.tags, d.name, c.mod
            FROM cards c
            JOIN notes n ON c.nid = n.id
            JOIN decks d ON c.did = d.id
            WHERE c.queue >= 0 AND (c.reps > 0 OR c.type = 2)
        ";

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, String>(9)?,
                row.get::<_, i64>(10)?,
            ))
        })?;

        let mut sentences = Vec::new();
        for item in rows {
            let row_data = item?;
            if let Some(sentence) =
                Self::parse_row(row_data, &fields_map, &rev_map, &paths.media_dir)
            {
                sentences.push(sentence);
            }
        }

        Ok(sentences)
    }

    fn parse_row(
        row: (
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            i64,
            String,
            String,
            String,
            i64,
        ),
        fields_map: &HashMap<i64, HashMap<usize, String>>,
        rev_map: &HashMap<i64, i64>,
        media_dir: &Path,
    ) -> Option<Sentence> {
        let (cid, nid, ivl, reps, _type, _queue, mid, flds, tags_str, deck, mod_time) = row;
        let empty_fields = HashMap::new();
        let field_names = fields_map.get(&mid).unwrap_or(&empty_fields);
        let fields: Vec<&str> = flds.split('\x1f').collect();

        let (japanese_text, furigana, translation, audio_file) =
            Self::extract_fields(&fields, field_names)?;

        let audio_path = media_dir.join(&audio_file);
        if !audio_path.exists() {
            return None;
        }

        let tags = tags_str.split_whitespace().map(|s| s.to_string()).collect();

        let status = if ivl >= 21 {
            CardStatus::Mature
        } else {
            CardStatus::Young
        };

        let last_reviewed = rev_map.get(&cid).copied().or(Some(mod_time * 1000));

        Some(Sentence {
            id: nid,
            card_id: cid,
            note_id: nid,
            japanese_text,
            furigana,
            translation,
            audio_path,
            deck,
            tags,
            status,
            interval: ivl,
            reps,
            last_reviewed,
            play_count: 0,
            shadow_count: 0,
            last_played: None,
        })
    }

    fn extract_fields(
        fields: &[&str],
        names: &HashMap<usize, String>,
    ) -> Option<(String, Option<String>, Option<String>, String)> {
        let mut jp_candidate = None;
        let mut furigana = None;
        let mut translation = None;
        let mut audio_candidates: Vec<(u8, String)> = Vec::new();

        for (ord, &val) in fields.iter().enumerate() {
            let name = names
                .get(&ord)
                .map(|s| s.to_lowercase())
                .unwrap_or_default();
            let cleaned = Self::strip_html(val);

            if Self::is_sound_field(val) {
                if let Some(sound) = Self::extract_sound_filename(val) {
                    let prio = if name.contains("sent") || name.contains("expression") {
                        2
                    } else if name.contains("vocab") || name.contains("word") {
                        1
                    } else {
                        0
                    };
                    audio_candidates.push((prio, sound));
                }
            }

            if name.contains("furigana") || name.contains("reading") {
                furigana = Some(cleaned.clone());
            } else if name.contains("eng")
                || name.contains("meaning")
                || name.contains("translation")
            {
                translation = Some(cleaned.clone());
            } else if jp_candidate.is_none() && Self::has_japanese(&cleaned) {
                jp_candidate = Some(cleaned);
            }
        }

        let japanese_text = jp_candidate?;
        audio_candidates.sort_by_key(|b| std::cmp::Reverse(b.0));
        let audio_file = audio_candidates.into_iter().next()?.1;

        Some((japanese_text, furigana, translation, audio_file))
    }

    fn strip_html(input: &str) -> String {
        let mut output = String::with_capacity(input.len());
        let mut inside_tag = false;

        for ch in input.chars() {
            match ch {
                '<' => inside_tag = true,
                '>' => inside_tag = false,
                _ if !inside_tag => output.push(ch),
                _ => {}
            }
        }

        output
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&#39;", "'")
            .replace("&quot;", "\"")
            .trim()
            .to_string()
    }

    fn has_japanese(text: &str) -> bool {
        text.chars().any(|c| {
            matches!(c,
                '\u{3040}'..='\u{309F}' | // Hiragana
                '\u{30A0}'..='\u{30FF}' | // Katakana
                '\u{4E00}'..='\u{9FFF}'   // CJK Ideographs
            )
        })
    }

    fn is_sound_field(val: &str) -> bool {
        val.contains("[sound:") || val.ends_with(".ogg") || val.ends_with(".mp3")
    }

    fn extract_sound_filename(val: &str) -> Option<String> {
        if let Some(start) = val.find("[sound:") {
            let rest = &val[start + 7..];
            if let Some(end) = rest.find(']') {
                return Some(rest[..end].to_string());
            }
        }
        None
    }
}
