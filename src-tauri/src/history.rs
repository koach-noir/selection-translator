use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY: usize = 20;
const MAX_SESSION: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TranslationEntry {
    pub id: u64,
    pub original: String,
    pub translated: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct TranslationHistory {
    entries: Vec<TranslationEntry>,
    next_id: u64,
}

impl TranslationHistory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
        }
    }

    /// イミュータブル: 新しい History と追加された Entry を返す
    pub fn add(&self, original: String, translated: String) -> (Self, TranslationEntry) {
        let entry = TranslationEntry {
            id: self.next_id,
            original,
            translated,
            timestamp: now_millis(),
        };

        let mut new_entries = self.entries.clone();
        new_entries.push(entry.clone());
        if new_entries.len() > MAX_HISTORY {
            new_entries.remove(0);
        }

        let new_history = Self {
            entries: new_entries,
            next_id: self.next_id + 1,
        };

        (new_history, entry)
    }

    pub fn recent(&self, n: usize) -> Vec<TranslationEntry> {
        let start = self.entries.len().saturating_sub(n);
        self.entries[start..].to_vec()
    }

    pub fn find_by_id(&self, id: u64) -> Option<&TranslationEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}

/// ポップアップセッション用: 最大5件を保持した新しい Vec を返す
pub fn append_to_session(
    session: &[TranslationEntry],
    entry: TranslationEntry,
) -> Vec<TranslationEntry> {
    let mut new_session: Vec<TranslationEntry> = session.to_vec();
    new_session.push(entry);
    if new_session.len() > MAX_SESSION {
        new_session.remove(0);
    }
    new_session
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_history_is_empty() {
        let history = TranslationHistory::new();
        assert_eq!(history.recent(5).len(), 0);
    }

    #[test]
    fn test_add_returns_new_history_and_entry() {
        let history = TranslationHistory::new();
        let (new_history, entry) = history.add("hello".into(), "こんにちは".into());

        assert_eq!(entry.id, 0);
        assert_eq!(entry.original, "hello");
        assert_eq!(entry.translated, "こんにちは");
        assert_eq!(new_history.recent(5).len(), 1);
        // 元の history は変更されていない
        assert_eq!(history.recent(5).len(), 0);
    }

    #[test]
    fn test_add_increments_id() {
        let h0 = TranslationHistory::new();
        let (h1, e1) = h0.add("a".into(), "A".into());
        let (_, e2) = h1.add("b".into(), "B".into());

        assert_eq!(e1.id, 0);
        assert_eq!(e2.id, 1);
    }

    #[test]
    fn test_max_history_overflow() {
        let mut history = TranslationHistory::new();
        for i in 0..25 {
            let (new_history, _) = history.add(format!("text{}", i), format!("翻訳{}", i));
            history = new_history;
        }

        let entries = history.recent(MAX_HISTORY + 5);
        assert_eq!(entries.len(), MAX_HISTORY);
        // 最古は id=5 (0〜4 は溢れて削除済み)
        assert_eq!(entries[0].id, 5);
        assert_eq!(entries[MAX_HISTORY - 1].id, 24);
    }

    #[test]
    fn test_recent_returns_last_n() {
        let mut history = TranslationHistory::new();
        for i in 0..10 {
            let (new_history, _) = history.add(format!("t{}", i), format!("T{}", i));
            history = new_history;
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].id, 7);
        assert_eq!(recent[2].id, 9);
    }

    #[test]
    fn test_recent_fewer_than_n() {
        let h0 = TranslationHistory::new();
        let (h1, _) = h0.add("a".into(), "A".into());

        let recent = h1.recent(5);
        assert_eq!(recent.len(), 1);
    }

    #[test]
    fn test_find_by_id() {
        let h0 = TranslationHistory::new();
        let (h1, _) = h0.add("hello".into(), "こんにちは".into());
        let (h2, _) = h1.add("world".into(), "世界".into());

        assert_eq!(h2.find_by_id(0).unwrap().original, "hello");
        assert_eq!(h2.find_by_id(1).unwrap().original, "world");
        assert!(h2.find_by_id(99).is_none());
    }

    #[test]
    fn test_append_to_session() {
        let session: Vec<TranslationEntry> = Vec::new();
        let entry = TranslationEntry {
            id: 0,
            original: "a".into(),
            translated: "A".into(),
            timestamp: 0,
        };

        let new_session = append_to_session(&session, entry);
        assert_eq!(new_session.len(), 1);
        // 元の session は変更されていない
        assert_eq!(session.len(), 0);
    }

    #[test]
    fn test_append_to_session_max_overflow() {
        let mut session = Vec::new();
        for i in 0..5 {
            session.push(TranslationEntry {
                id: i,
                original: format!("t{}", i),
                translated: format!("T{}", i),
                timestamp: 0,
            });
        }

        let new_entry = TranslationEntry {
            id: 5,
            original: "t5".into(),
            translated: "T5".into(),
            timestamp: 0,
        };

        let new_session = append_to_session(&session, new_entry);
        assert_eq!(new_session.len(), MAX_SESSION);
        assert_eq!(new_session[0].id, 1); // id=0 が削除されている
        assert_eq!(new_session[4].id, 5);
    }
}
