//! The command language shared by key bindings, the CLI/IPC socket and the
//! overlay UI: `focus left`, `workspace 3`, `grid apply 0,0 3x4 / 6x4`.
//!
//! Commands are registered by features under a name. Several commands can be
//! chained with `;`: `float on; center`.

use crate::engine::Engine;
use crate::geometry::Direction;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::str::FromStr;

/// `Ok(Some(text))` is printed by the CLI.
pub type CmdResult = Result<Option<String>, String>;
pub type Handler = Rc<dyn Fn(&mut Engine, &Args) -> CmdResult>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    pub name: String,
    pub words: Vec<String>,
}

impl Args {
    pub fn get(&self, i: usize) -> Option<&str> {
        self.words.get(i).map(|s| s.as_str())
    }

    pub fn require(&self, i: usize, what: &str) -> Result<&str, String> {
        self.get(i).ok_or_else(|| format!("`{}` needs {what}", self.name))
    }

    pub fn parse<T: FromStr>(&self, i: usize, what: &str) -> Result<T, String>
    where
        T::Err: std::fmt::Display,
    {
        let s = self.require(i, what)?;
        s.parse::<T>().map_err(|e| format!("`{}`: bad {what} `{s}`: {e}", self.name))
    }

    pub fn direction(&self, i: usize) -> Result<Direction, String> {
        self.parse(i, "a direction (left/right/up/down)")
    }

    pub fn has_flag(&self, flag: &str) -> bool {
        self.words.iter().any(|w| w == flag)
    }

    /// Everything after position `i`, re-joined (for `exec`).
    pub fn rest(&self, i: usize) -> String {
        self.words.get(i..).map(|w| w.join(" ")).unwrap_or_default()
    }

    /// Positional words with `--flags` removed.
    pub fn positional(&self) -> Vec<&str> {
        self.words.iter().filter(|w| !w.starts_with("--")).map(|s| s.as_str()).collect()
    }
}

/// Split a command line into commands and their words. Supports single and
/// double quotes and `;` as a separator.
pub fn parse_line(line: &str) -> Result<Vec<Args>, String> {
    let mut commands = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_word = false;
    let mut quote: Option<char> = None;
    let push_cmd = |words: &mut Vec<String>, commands: &mut Vec<Args>| {
        if !words.is_empty() {
            let name = words.remove(0);
            commands.push(Args { name, words: std::mem::take(words) });
        }
    };
    for c in line.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                in_word = true;
            }
            (None, ';') => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
                push_cmd(&mut words, &mut commands);
            }
            (None, c) if c.is_whitespace() => {
                if in_word {
                    words.push(std::mem::take(&mut cur));
                    in_word = false;
                }
            }
            (None, c) => {
                cur.push(c);
                in_word = true;
            }
        }
    }
    if quote.is_some() {
        return Err(format!("unterminated quote in `{line}`"));
    }
    if in_word {
        words.push(cur);
    }
    push_cmd(&mut words, &mut commands);
    Ok(commands)
}

#[derive(Clone)]
pub struct CommandSpec {
    pub name: String,
    pub usage: String,
    pub summary: String,
    pub feature: String,
    pub handler: Handler,
}

impl std::fmt::Debug for CommandSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CommandSpec").field("name", &self.name).field("feature", &self.feature).finish()
    }
}

#[derive(Debug, Clone, Default)]
pub struct CommandRegistry {
    specs: BTreeMap<String, CommandSpec>,
}

impl CommandRegistry {
    pub fn insert(&mut self, spec: CommandSpec) {
        self.specs.insert(spec.name.clone(), spec);
    }

    pub fn get(&self, name: &str) -> Option<&CommandSpec> {
        self.specs.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &CommandSpec> {
        self.specs.values()
    }

    pub fn len(&self) -> usize {
        self.specs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.specs.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_words_quotes_and_chains() {
        let cmds = parse_line(r#"float on; exec open -a "Visual Studio Code";center"#).unwrap();
        assert_eq!(cmds.len(), 3);
        assert_eq!(cmds[0], Args { name: "float".into(), words: vec!["on".into()] });
        assert_eq!(cmds[1].words, vec!["open", "-a", "Visual Studio Code"]);
        assert_eq!(cmds[2].name, "center");
        assert!(parse_line("exec 'oops").is_err());
        assert!(parse_line("   ").unwrap().is_empty());
    }
}
