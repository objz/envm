use std::env;
use std::io::{self, IsTerminal};
use std::sync::OnceLock;

struct Palette {
    enabled: bool,
}

impl Palette {
    fn new() -> Self {
        let no_color = env::var_os("NO_COLOR").is_some();
        let enabled = !no_color && io::stderr().is_terminal();
        Self { enabled }
    }
    fn paint(&self, s: &str, code: &str) -> String {
        if self.enabled {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
    fn bold(&self, s: &str) -> String {
        self.paint(s, "1")
    }
    fn red(&self, s: &str) -> String {
        self.paint(s, "31")
    }
    fn yellow(&self, s: &str) -> String {
        self.paint(s, "33")
    }
}

fn palette() -> &'static Palette {
    static INSTANCE: OnceLock<Palette> = OnceLock::new();
    INSTANCE.get_or_init(Palette::new)
}

pub fn success(msg: impl AsRef<str>) {
    let p = palette();
    println!("{} {}", p.paint("✔", "32"), msg.as_ref());
}

pub fn warn(msg: impl AsRef<str>) {
    let p = palette();
    eprintln!("{} {}", p.yellow("⚠"), msg.as_ref());
}

pub fn error(msg: impl AsRef<str>) {
    let p = palette();
    eprintln!("{} {}", p.red(&p.bold("✖")), msg.as_ref());
}

pub fn kv_listing(header_path: &str, items: &[(String, String)]) {
    if items.is_empty() {
        println!("# {} (no variables)", header_path);
        return;
    }
    println!("# {}", header_path);
    let width = items.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (k, v) in items {
        println!("{:width$} = {}", k, v, width = width);
    }
}
