//! The "Add a feature" example from docs/ARCHITECTURE.md, kept compiling.

use gridpilot_core::config::Config;
use gridpilot_core::features::builtin;
use gridpilot_core::geometry::Rect;
use gridpilot_core::testing::MockPlatform;
use gridpilot_core::{Engine, EngineEvent, Feature, Registrar};

pub struct AutoCenterDialogs;

impl Feature for AutoCenterDialogs {
    fn id(&self) -> &'static str {
        "auto-center"
    }
    fn summary(&self) -> &'static str {
        "Centre new floating windows"
    }

    fn install(&self, r: &mut Registrar<'_>) {
        r.command("center-all", "center-all", "Centre every floating window", |e, _| {
            let ws = e.world.current_workspace().ok_or("no workspace")?;
            let area = e.workspace_area(&ws);
            for id in e.world.workspaces[&ws].floating.clone() {
                let w = e.world.windows.get_mut(&id).unwrap();
                w.float_frame = area.centered(w.float_frame.size());
            }
            Ok(None)
        });
        r.bind("mod+shift+c", "center-all", "Centre floating windows");
        r.on(|e: &mut Engine, ev| {
            if let EngineEvent::WindowAdded(id) = ev {
                if e.world.windows[id].floating {
                    let _ = e.run("center-all");
                }
            }
        });
    }
}

#[test]
fn custom_feature_plugs_in() {
    let p = MockPlatform::with_displays(1);
    let mut features = builtin();
    features.push(Box::new(AutoCenterDialogs));
    let config =
        Config::parse("[[rule]]\napp = \"calc\"\nfloat = true\n[tiling]\ndefault_layout = \"dwindle\"").unwrap();
    let mut e = Engine::new(Box::new(p.clone()), config, features);
    e.start().unwrap();
    let w = p.open(1, "calc", Rect::new(0.0, 0.0, 400.0, 300.0));
    e.refresh().unwrap();
    assert_eq!(p.state().frame(w), Rect::new(760.0, 410.0, 400.0, 300.0));
    assert!(e.installed_features().iter().any(|(id, _)| *id == "auto-center"));
}
