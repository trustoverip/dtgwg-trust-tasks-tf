//! `trust-tasks.toml` — the application's intent.
//!
//! ```toml
//! registry = "https://trusttasks.org"   # optional, default
//! [tasks]
//! "vault/upsert" = "0.4"        # exact version
//! "acl/grant" = "latest"        # the latest live version, at add/update time
//! ```
//!
//! A value may also be an array of exact versions (`["0.3", "0.4"]`) for an
//! application that serves two versions of a task at once.
//!
//! Edited in place with `toml_edit`, so the author's comments survive.

use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use toml_edit::{Array, DocumentMut, Item, Table, Value};

use crate::names::{check_slug, Version};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Intent {
    Exact(Version),
    Latest,
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Intent::Exact(v) => write!(f, "{v}"),
            Intent::Latest => f.write_str("latest"),
        }
    }
}

pub struct Manifest {
    pub path: PathBuf,
    doc: DocumentMut,
    /// Parsed `[tasks]`, kept in step with `doc`.
    pub tasks: BTreeMap<String, Vec<Intent>>,
}

const NEW_MANIFEST: &str = "\
# The Trust Task specifications this application uses. `trust-tasks add`,
# `remove` and `update` edit this file; `trust-tasks.lock` records what each
# entry resolved to. A value is an exact version (\"0.4\") or \"latest\".
[tasks]
";

impl Manifest {
    pub fn new(path: PathBuf) -> Manifest {
        Manifest {
            path,
            doc: NEW_MANIFEST.parse().expect("the template parses"),
            tasks: BTreeMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Manifest> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let doc: DocumentMut = text
            .parse()
            .with_context(|| format!("{} is not valid TOML", path.display()))?;
        let tasks = parse_tasks(&doc).with_context(|| format!("in {}", path.display()))?;
        Ok(Manifest {
            path: path.to_path_buf(),
            doc,
            tasks,
        })
    }

    pub fn registry(&self) -> Result<Option<String>> {
        match self.doc.get("registry") {
            None => Ok(None),
            Some(item) => item
                .as_str()
                .map(|s| Some(s.to_string()))
                .ok_or_else(|| anyhow!("{}: `registry` must be a string", self.path.display())),
        }
    }

    fn table(&mut self) -> &mut Table {
        if !self.doc.contains_table("tasks") {
            self.doc.insert("tasks", Item::Table(Table::new()));
        }
        self.doc["tasks"]
            .as_table_mut()
            .expect("[tasks] is a table")
    }

    /// Sets `slug`'s intents (sorted, deduplicated), replacing what was there.
    pub fn set(&mut self, slug: &str, mut intents: Vec<Intent>) {
        intents.sort();
        intents.dedup();
        let value = match intents.as_slice() {
            [one] => Value::from(one.to_string()),
            many => Value::Array(many.iter().map(|i| i.to_string()).collect::<Array>()),
        };
        let table = self.table();
        match table.get_mut(slug).and_then(Item::as_value_mut) {
            // Keep the entry's decor — a trailing comment, say.
            Some(existing) => {
                let decor = existing.decor().clone();
                *existing = value;
                *existing.decor_mut() = decor;
            }
            None => {
                table.insert(slug, Item::Value(value));
            }
        }
        table.sort_values();
        self.tasks.insert(slug.to_string(), intents);
    }

    pub fn remove(&mut self, slug: &str) {
        self.table().remove(slug);
        self.tasks.remove(slug);
    }

    pub fn render(&self) -> String {
        self.doc.to_string()
    }
}

fn parse_tasks(doc: &DocumentMut) -> Result<BTreeMap<String, Vec<Intent>>> {
    let mut out = BTreeMap::new();
    let Some(item) = doc.get("tasks") else {
        return Ok(out);
    };
    let table = item
        .as_table_like()
        .ok_or_else(|| anyhow!("`tasks` must be a table of slug = version"))?;
    for (slug, item) in table.iter() {
        check_slug(slug)?;
        let bad = || {
            anyhow!("tasks.\"{slug}\": expected \"latest\", a version such as \"0.4\", or an array of versions")
        };
        let intents = match item.as_value().ok_or_else(bad)? {
            Value::String(s) => vec![parse_intent(s.value()).ok_or_else(bad)?],
            Value::Array(arr) => {
                let mut v = Vec::new();
                for x in arr.iter() {
                    let s = x.as_str().ok_or_else(bad)?;
                    match parse_intent(s).ok_or_else(bad)? {
                        Intent::Latest => bail!(
                            "tasks.\"{slug}\": \"latest\" cannot be one of several versions — it already names exactly one"
                        ),
                        i => v.push(i),
                    }
                }
                if v.is_empty() {
                    return Err(bad());
                }
                v.sort();
                v.dedup();
                v
            }
            _ => return Err(bad()),
        };
        out.insert(slug.to_string(), intents);
    }
    Ok(out)
}

fn parse_intent(s: &str) -> Option<Intent> {
    if s == "latest" {
        Some(Intent::Latest)
    } else {
        Version::parse(s).map(Intent::Exact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_keep_comments_and_sort() {
        let text = "# mine\nregistry = \"file:///r\"\n[tasks]\n\"vault/upsert\" = \"0.4\" # pinned for the demo\n";
        let mut m = Manifest {
            path: "x".into(),
            doc: text.parse().unwrap(),
            tasks: parse_tasks(&text.parse().unwrap()).unwrap(),
        };
        m.set("acl/grant", vec![Intent::Latest]);
        m.set(
            "vault/upsert",
            vec![Intent::Exact(Version::parse("0.3").unwrap())],
        );
        let out = m.render();
        assert!(out.starts_with("# mine\n"), "{out}");
        assert!(out.contains("# pinned for the demo"), "{out}");
        let acl = out.find("acl/grant").unwrap();
        let vault = out.find("vault/upsert").unwrap();
        assert!(acl < vault, "{out}");
        assert!(out.contains("\"vault/upsert\" = \"0.3\""), "{out}");
    }

    #[test]
    fn arrays_and_errors() {
        let doc: DocumentMut = "[tasks]\n\"a/b\" = [\"0.4\", \"0.3\"]\n".parse().unwrap();
        let t = parse_tasks(&doc).unwrap();
        assert_eq!(t["a/b"].len(), 2);
        let doc: DocumentMut = "[tasks]\n\"a/b\" = [\"latest\", \"0.3\"]\n"
            .parse()
            .unwrap();
        assert!(parse_tasks(&doc).is_err());
        let doc: DocumentMut = "[tasks]\n\"a/b\" = 4\n".parse().unwrap();
        assert!(parse_tasks(&doc).is_err());
    }
}
