//! Keys a file may still contain although the schema has moved on. Unknown keys are errors, and a file with
//! an error is not loaded at all: without this, renaming or removing an option would cost everyone who has
//! set it their whole configuration at the next update. So such a key is listed here instead, taken out of
//! the file's table before it is read, and reported, and the rest of the file loads as before.
use toml::{Table, Value};

#[cfg_attr(not(test), expect(dead_code, reason = "OUTDATED is empty at the moment"))]
pub(crate) enum Fate {
    /// The key has this name now, in the same table; its value is kept unless the new key is set as well.
    Renamed(&'static str),
    /// The option is gone; its value is ignored.
    Removed,
}

/// Every key that was renamed or removed, as its dotted path (`appearance.gaps`). Of the path's tables, an
/// array of tables stands for each of its entries (`rules.opacity`), and `*` for every table whatever its
/// name (`program_bindings.*.keys`). Add the date, so that an entry can go once nobody has the key any more.
pub(crate) const OUTDATED: &[(&str, Fate)] = &[];

/// Renames and removes the `outdated` keys of `table`. Returns what to tell the user, one line per key found.
pub(crate) fn apply(table: &mut Table, outdated: &[(&str, Fate)]) -> Vec<String> {
    let mut notes = Vec::new();
    for (path, fate) in outdated {
        let mut segments: Vec<&str> = path.split('.').collect();
        let Some(key) = segments.pop() else { continue };
        if visit(table, &segments, key, fate) {
            notes.push(match fate {
                Fate::Renamed(new) => format!("{path} is now called {new}: rename it"),
                Fate::Removed => format!("{path} no longer exists: remove it"),
            });
        }
    }
    notes
}

/// Whether `key` was in `table` or, further down `parents`, in one of its tables.
fn visit(table: &mut Table, parents: &[&str], key: &str, fate: &Fate) -> bool {
    let Some((first, rest)) = parents.split_first() else {
        let Some(value) = table.remove(key) else { return false };
        if let Fate::Renamed(new) = fate
            && !table.contains_key(*new)
        {
            table.insert((*new).to_owned(), value);
        }
        return true;
    };
    let mut found = false;
    let children = table.iter_mut().filter(|(name, _)| *first == "*" || name.as_str() == *first).map(|(_, value)| value);
    for child in children {
        match child {
            Value::Table(table) => found |= visit(table, rest, key, fate),
            Value::Array(entries) => {
                for table in entries.iter_mut().filter_map(Value::as_table_mut) {
                    found |= visit(table, rest, key, fate);
                }
            }
            _ => {}
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table(text: &str) -> Table {
        toml::from_str(text).unwrap()
    }

    #[test]
    fn a_renamed_key_keeps_its_value_under_the_new_name() {
        let mut file = table("[appearance]\ngaps = 4\n");
        let notes = apply(&mut file, &[("appearance.gaps", Fate::Renamed("gaps_inner"))]);
        assert_eq!(file, table("[appearance]\ngaps_inner = 4\n"));
        assert_eq!(notes, ["appearance.gaps is now called gaps_inner: rename it"]);
    }

    #[test]
    fn the_new_name_wins_over_the_old_one() {
        let mut file = table("[appearance]\ngaps = 4\ngaps_inner = 9\n");
        assert_eq!(apply(&mut file, &[("appearance.gaps", Fate::Renamed("gaps_inner"))]).len(), 1);
        assert_eq!(file, table("[appearance]\ngaps_inner = 9\n"));
    }

    #[test]
    fn a_removed_key_goes_and_is_named_once() {
        let mut file =
            table("old = true\n[[rules]]\napp_id = 'a'\nold = 1\n[[rules]]\nold = 2\n[bindings.x]\nold = 3\n[bindings.y]\nkeys = []\n");
        let outdated = [("old", Fate::Removed), ("rules.old", Fate::Removed), ("bindings.*.old", Fate::Removed)];
        assert_eq!(
            apply(&mut file, &outdated),
            ["old no longer exists: remove it", "rules.old no longer exists: remove it", "bindings.*.old no longer exists: remove it"]
        );
        assert_eq!(file, table("[[rules]]\napp_id = 'a'\n[[rules]]\n[bindings.x]\n[bindings.y]\nkeys = []\n"));
    }

    #[test]
    fn a_file_without_outdated_keys_stays_as_it_is() {
        let mut file = table("terminal = ['foot']\n[appearance]\ngaps_inner = 4\n");
        let before = file.clone();
        let outdated = [("appearance.gaps", Fate::Renamed("gaps_inner")), ("terminal.x", Fate::Removed), ("missing.*.y", Fate::Removed)];
        assert!(apply(&mut file, &outdated).is_empty());
        assert_eq!(file, before);
    }

    #[test]
    fn the_list_is_well_formed() {
        for (path, fate) in OUTDATED {
            assert!(!path.is_empty() && path.split('.').all(|segment| !segment.is_empty()), "{path}");
            assert!(!path.ends_with('*'), "{path}: the key itself needs a name");
            if let Fate::Renamed(new) = fate {
                assert!(!new.is_empty() && !new.contains('.'), "{path}: a key is renamed within its table");
            }
        }
    }
}
