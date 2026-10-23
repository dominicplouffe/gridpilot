//! Features are how GridPilot grows.
//!
//! Each feature (the Divvy grid, focus navigation, workspaces, the scratchpad,
//! groups, ...) is a self-contained module that, when installed, registers:
//!
//! * commands (`focus`, `workspace`, `grid`, ...),
//! * default key bindings for those commands,
//! * layouts, and
//! * hooks that react to [`EngineEvent`]s.
//!
//! Users can switch features off in the config (`[features] disabled = [...]`)
//! and their commands and bindings disappear with them. A new behaviour from
//! Omarchy, Divvy or anywhere else is one more `impl Feature`.

use crate::command::{Args, CmdResult, CommandRegistry, CommandSpec};
use crate::config::Config;
use crate::engine::Engine;
use crate::layout::{LayoutCtor, LayoutRegistry};
use crate::platform::{DisplayId, WindowId};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq)]
pub enum EngineEvent {
    WindowAdded(WindowId),
    WindowRemoved(WindowId),
    FocusChanged(Option<WindowId>),
    WorkspaceShown { display: DisplayId, name: String },
    LayoutChanged { workspace: String, layout: String },
    ConfigReloaded,
}

pub type Hook = Rc<dyn Fn(&mut Engine, &EngineEvent)>;

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultBinding {
    pub chord: String,
    pub command: String,
    pub description: String,
    pub feature: String,
    /// Bindings generated from user config (grid shortcuts) are kept even
    /// when `general.default_bindings = false`.
    pub from_config: bool,
}

pub trait Feature {
    /// Stable id used in `[features] disabled`.
    fn id(&self) -> &'static str;
    fn summary(&self) -> &'static str;
    fn install(&self, r: &mut Registrar<'_>);
}

pub struct Registrar<'a> {
    pub(crate) feature: &'static str,
    pub(crate) config: &'a Config,
    pub(crate) commands: &'a mut CommandRegistry,
    pub(crate) bindings: &'a mut Vec<DefaultBinding>,
    pub(crate) layouts: &'a mut LayoutRegistry,
    pub(crate) hooks: &'a mut Vec<(String, Hook)>,
}

impl Registrar<'_> {
    pub fn config(&self) -> &Config {
        self.config
    }

    pub fn command(
        &mut self,
        name: &str,
        usage: &str,
        summary: &str,
        handler: impl Fn(&mut Engine, &Args) -> CmdResult + 'static,
    ) {
        self.commands.insert(CommandSpec {
            name: name.into(),
            usage: usage.into(),
            summary: summary.into(),
            feature: self.feature.into(),
            handler: Rc::new(handler),
        });
    }

    pub fn bind(&mut self, chord: &str, command: &str, description: &str) {
        self.push_binding(chord, command, description, false);
    }

    pub fn bind_from_config(&mut self, chord: &str, command: &str, description: &str) {
        self.push_binding(chord, command, description, true);
    }

    fn push_binding(&mut self, chord: &str, command: &str, description: &str, from_config: bool) {
        self.bindings.push(DefaultBinding {
            chord: chord.into(),
            command: command.into(),
            description: description.into(),
            feature: self.feature.into(),
            from_config,
        });
    }

    pub fn layout(&mut self, name: &str, ctor: LayoutCtor) {
        self.layouts.register(name, ctor);
    }

    pub fn on(&mut self, hook: impl Fn(&mut Engine, &EngineEvent) + 'static) {
        self.hooks.push((self.feature.into(), Rc::new(hook)));
    }
}
