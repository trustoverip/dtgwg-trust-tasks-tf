//! `trust-tasks.toml` — the application's intent.
//!
//! ```toml
//! registry = "https://trusttasks.org"   # optional, default
//!
//! [sources.acme]                        # a third-party source (SPEC §6.5)
//! url = "https://tasks.acme.example"    # or: path = "../acme-registry"
//!
//! [tasks]
//! "vault/upsert" = "0.4"                # exact version
//! "acl/grant" = "latest"                # the latest live version, at add/update time
//! "acme:payments/settle" = "1.0"        # a task from the source `acme`
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

use crate::names::{check_source_name, TaskId, Version};

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

/// Where a third-party source's registry is: an `https://` site root, or a
/// directory laid out the same way, relative to the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceLoc {
    Url(String),
    Path(String),
}

impl fmt::Display for SourceLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SourceLoc::Url(u) => write!(f, "url {u}"),
            SourceLoc::Path(p) => write!(f, "path {p}"),
        }
    }
}

/// Validates a source's `url`: https only. A source names specifications an
/// application will generate code for; over plain http anyone on the path
/// could choose them, and SPEC §6.1 forbids http Type URIs for the same
/// reason.
pub fn check_source_url(name: &str, url: &str) -> Result<()> {
    let scheme = url.split_once("://").map(|(s, _)| s);
    match scheme {
        Some(s) if s.eq_ignore_ascii_case("https") && url.len() > "https://".len() => Ok(()),
        Some(s) if s.eq_ignore_ascii_case("http") => bail!(
            "[sources.{name}]: url {url} is http — a source must be https://, because over http anyone on the network path could choose the specifications you generate code for (and SPEC §6.1 forbids http)"
        ),
        _ => bail!(
            "[sources.{name}]: url {url:?} is not an https:// URL (for a directory, write `path = \"…\"`)"
        ),
    }
}

pub struct Manifest {
    pub path: PathBuf,
    doc: DocumentMut,
    /// Parsed `[tasks]`, kept in step with `doc`.
    pub tasks: BTreeMap<TaskId, Vec<Intent>>,
    /// Parsed `[sources]`.
    pub sources: BTreeMap<String, SourceLoc>,
}

const NEW_MANIFEST: &str = "\
# The Trust Task specifications this application uses. `trust-tasks add`,
# `remove` and `update` edit this file; `trust-tasks.lock` records what each
# entry resolved to. A value is an exact version (\"0.4\") or \"latest\".
# Tasks from a third-party source are named \"<source>:<slug>\", with the
# source declared as [sources.<source>] url = \"https://…\" (or path = \"…\").
[tasks]
";

impl Manifest {
    pub fn new(path: PathBuf) -> Manifest {
        Manifest {
            path,
            doc: NEW_MANIFEST.parse().expect("the template parses"),
            tasks: BTreeMap::new(),
            sources: BTreeMap::new(),
        }
    }

    pub fn load(path: &Path) -> Result<Manifest> {
        let text =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let doc: DocumentMut = text
            .parse()
            .with_context(|| format!("{} is not valid TOML", path.display()))?;
        let sources = parse_sources(&doc).with_context(|| format!("in {}", path.display()))?;
        let tasks =
            parse_tasks(&doc, &sources).with_context(|| format!("in {}", path.display()))?;
        Ok(Manifest {
            path: path.to_path_buf(),
            doc,
            tasks,
            sources,
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

    /// The declared source `name`, or an error saying how to declare it.
    pub fn source(&self, name: &str) -> Result<&SourceLoc> {
        self.sources.get(name).ok_or_else(|| {
            anyhow!(
                "source `{name}` is not declared in {} — add\n\n    [sources.{name}]\n    url = \"https://…\"   # or: path = \"<dir>\"\n",
                self.path.display()
            )
        })
    }

    fn table(&mut self) -> &mut Table {
        if !self.doc.contains_table("tasks") {
            self.doc.insert("tasks", Item::Table(Table::new()));
        }
        self.doc["tasks"]
            .as_table_mut()
            .expect("[tasks] is a table")
    }

    /// Sets `id`'s intents (sorted, deduplicated), replacing what was there.
    pub fn set(&mut self, id: &TaskId, mut intents: Vec<Intent>) {
        intents.sort();
        intents.dedup();
        let value = match intents.as_slice() {
            [one] => Value::from(one.to_string()),
            many => Value::Array(many.iter().map(|i| i.to_string()).collect::<Array>()),
        };
        let key = id.to_string();
        let table = self.table();
        match table.get_mut(&key).and_then(Item::as_value_mut) {
            // Keep the entry's decor — a trailing comment, say.
            Some(existing) => {
                let decor = existing.decor().clone();
                *existing = value;
                *existing.decor_mut() = decor;
            }
            None => {
                table.insert(&key, Item::Value(value));
            }
        }
        table.sort_values();
        self.tasks.insert(id.clone(), intents);
    }

    pub fn remove(&mut self, id: &TaskId) {
        self.table().remove(&id.to_string());
        self.tasks.remove(id);
    }

    pub fn render(&self) -> String {
        self.doc.to_string()
    }
}

fn parse_sources(doc: &DocumentMut) -> Result<BTreeMap<String, SourceLoc>> {
    let mut out = BTreeMap::new();
    let Some(item) = doc.get("sources") else {
        return Ok(out);
    };
    let table = item
        .as_table_like()
        .ok_or_else(|| anyhow!("`sources` must be a table of [sources.<name>] tables"))?;
    for (name, item) in table.iter() {
        check_source_name(name).with_context(|| format!("[sources.{name}]"))?;
        let src = item
            .as_table_like()
            .ok_or_else(|| anyhow!("[sources.{name}] must be a table with `url` or `path`"))?;
        let mut url = None;
        let mut path = None;
        for (key, value) in src.iter() {
            let text = value
                .as_str()
                .ok_or_else(|| anyhow!("[sources.{name}]: `{key}` must be a string"))?;
            match key {
                "url" => url = Some(text.to_string()),
                "path" => path = Some(text.to_string()),
                other => {
                    bail!("[sources.{name}]: unknown key `{other}` (expected `url` or `path`)")
                }
            }
        }
        let loc = match (url, path) {
            (Some(u), None) => {
                check_source_url(name, &u)?;
                SourceLoc::Url(u.trim_end_matches('/').to_string())
            }
            (None, Some(p)) if !p.is_empty() => SourceLoc::Path(p),
            _ => bail!("[sources.{name}]: give exactly one of `url = \"https://…\"` and `path = \"<dir>\"`"),
        };
        out.insert(name.to_string(), loc);
    }
    Ok(out)
}

fn parse_tasks(
    doc: &DocumentMut,
    sources: &BTreeMap<String, SourceLoc>,
) -> Result<BTreeMap<TaskId, Vec<Intent>>> {
    let mut out = BTreeMap::new();
    let Some(item) = doc.get("tasks") else {
        return Ok(out);
    };
    let table = item
        .as_table_like()
        .ok_or_else(|| anyhow!("`tasks` must be a table of slug = version"))?;
    for (key, item) in table.iter() {
        let id = TaskId::parse(key)?;
        if let Some(source) = &id.source {
            if !sources.contains_key(source) {
                bail!(
                    "tasks.\"{key}\": source `{source}` is not declared — add [sources.{source}] with `url` or `path`"
                );
            }
        }
        let bad = || {
            anyhow!("tasks.\"{key}\": expected \"latest\", a version such as \"0.4\", or an array of versions")
        };
        let intents = match item.as_value().ok_or_else(bad)? {
            Value::String(s) => vec![parse_intent(s.value()).ok_or_else(bad)?],
            Value::Array(arr) => {
                let mut v = Vec::new();
                for x in arr.iter() {
                    let s = x.as_str().ok_or_else(bad)?;
                    match parse_intent(s).ok_or_else(bad)? {
                        Intent::Latest => bail!(
                            "tasks.\"{key}\": \"latest\" cannot be one of several versions — it already names exactly one"
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
        out.insert(id, intents);
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

    fn load(text: &str) -> Result<Manifest> {
        let doc: DocumentMut = text.parse().unwrap();
        let sources = parse_sources(&doc)?;
        let tasks = parse_tasks(&doc, &sources)?;
        Ok(Manifest {
            path: "x".into(),
            doc,
            tasks,
            sources,
        })
    }

    fn id(s: &str) -> TaskId {
        TaskId::parse(s).unwrap()
    }

    #[test]
    fn edits_keep_comments_and_sort() {
        let text = "# mine\nregistry = \"file:///r\"\n[tasks]\n\"vault/upsert\" = \"0.4\" # pinned for the demo\n";
        let mut m = load(text).unwrap();
        m.set(&id("acl/grant"), vec![Intent::Latest]);
        m.set(
            &id("vault/upsert"),
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
        let t = load("[tasks]\n\"a/b\" = [\"0.4\", \"0.3\"]\n")
            .unwrap()
            .tasks;
        assert_eq!(t[&id("a/b")].len(), 2);
        assert!(load("[tasks]\n\"a/b\" = [\"latest\", \"0.3\"]\n").is_err());
        assert!(load("[tasks]\n\"a/b\" = 4\n").is_err());
    }

    #[test]
    fn sources_are_declared_https_or_paths() {
        let m = load(
            "[sources.acme]\nurl = \"https://tasks.acme.example/\"\n[sources.local]\npath = \"../reg\"\n\
             [tasks]\n\"acme:payments/settle\" = \"1.0\"\n\"local:x\" = \"latest\"\n",
        )
        .unwrap();
        assert_eq!(
            m.sources["acme"],
            SourceLoc::Url("https://tasks.acme.example".into())
        );
        assert_eq!(m.sources["local"], SourceLoc::Path("../reg".into()));
        assert!(m.tasks.contains_key(&id("acme:payments/settle")));

        let err = |text: &str| format!("{:#}", load(text).err().expect(text));
        assert!(err("[sources.acme]\nurl = \"http://tasks.acme.example\"\n")
            .contains("must be https://"));
        assert!(err("[sources.acme]\nurl = \"ftp://x\"\n").contains("not an https:// URL"));
        assert!(err("[sources.acme]\nurl = \"https://a\"\npath = \"b\"\n").contains("exactly one"));
        assert!(err("[sources.acme]\n").contains("exactly one"));
        assert!(
            err("[sources.acme]\nurl = \"https://a\"\nmirror = \"b\"\n").contains("unknown key")
        );
        assert!(err("[sources.Acme]\nurl = \"https://a\"\n").contains("not a source name"));
        assert!(err("[tasks]\n\"acme:x\" = \"1.0\"\n").contains("not declared"));
        assert!(err(
            "[sources.acme]\nurl = \"https://a\"\n[tasks]\n\"acme:trust-task-ok\" = \"0.1\"\n"
        )
        .contains("framework slug"));
    }
}
