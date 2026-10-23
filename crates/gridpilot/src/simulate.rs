//! `gridpilot simulate`: the real engine and features on a fake desktop with
//! two displays, driven from stdin. Handy for trying commands, developing
//! features on any OS, and demos.

use gridpilot_core::features::builtin;
use gridpilot_core::geometry::Rect;
use gridpilot_core::testing::MockPlatform;
use gridpilot_core::{Config, Engine};
use std::io::{BufRead, Write};

pub fn run(config: Config) -> Result<(), String> {
    let p = MockPlatform::with_displays(2);
    let mut e = Engine::new(Box::new(p.clone()), config, builtin());
    e.start()?;
    let mut next_id = 1;
    println!("GridPilot simulator: 2 displays (1920x1080). Type commands, or:");
    println!("  open [app]   add a window      close <id>   close a window");
    println!("  show         print window frames            help / quit");
    let stdin = std::io::stdin();
    loop {
        print!("gridpilot> ");
        std::io::stdout().flush().ok();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            break;
        }
        let line = line.trim();
        let mut words = line.split_whitespace();
        match words.next() {
            None => continue,
            Some("exit") => break,
            Some("help") => println!("{}", e.dispatch("commands").unwrap_or_default().unwrap_or_default()),
            Some("open") => {
                let app = words.next().unwrap_or("app");
                p.open(next_id, app, Rect::new(200.0 + 30.0 * next_id as f64, 150.0, 900.0, 600.0));
                next_id += 1;
                e.refresh()?;
                show(&e, &p);
            }
            Some("close") => {
                let id: u64 = words.next().and_then(|w| w.parse().ok()).ok_or("close <id>")?;
                p.close_window(gridpilot_core::WindowId(id));
                e.refresh()?;
                show(&e, &p);
            }
            Some("show") => show(&e, &p),
            Some(_) => {
                match e.dispatch(line) {
                    Ok(Some(out)) => println!("{out}"),
                    Ok(None) => show(&e, &p),
                    Err(err) => println!("error: {err}"),
                }
                if e.quit_requested {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn show(e: &Engine, p: &MockPlatform) {
    let s = p.state();
    for w in &s.windows {
        let Some(m) = e.world.windows.get(&w.id) else { continue };
        let state = if s.is_hidden(w.id) { "hidden".to_string() } else { w.frame.to_string() };
        let focus = if e.world.focused == Some(w.id) { "*" } else { " " };
        println!(
            "{focus} [{}] {:<10} ws {:<10} {:<9} {}",
            w.id,
            w.app,
            m.workspace,
            if m.floating { "floating" } else { "tiled" },
            state
        );
    }
    let shown: Vec<String> = e.world.active.iter().map(|(d, ws)| format!("display {d}: {ws}")).collect();
    println!("  ({})", shown.join(", "));
}
