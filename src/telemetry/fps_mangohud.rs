use std::fs;
use std::path::Path;

pub struct MangoHudFpsReader {
    enabled: bool,
}

impl MangoHudFpsReader {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub fn poll(&self) -> Option<(f32, f32)> {
        if !self.enabled {
            return None;
        }

        // Look for MangoHud / MangoApp socket or output file in /tmp
        let candidates = ["/tmp/mangohud", "/tmp/mangoapp", "/tmp/mangohud_pipe"];
        for path_str in candidates {
            let p = Path::new(path_str);
            if p.exists() {
                if let Ok(content) = fs::read_to_string(p) {
                    if let Some(last_line) = content.lines().last() {
                        let trimmed = last_line.trim();
                        if trimmed.contains(',') {
                            let parts: Vec<&str> = trimmed.split(',').collect();
                            if let (Ok(fps), Ok(lat)) = (parts[0].parse::<f32>(), parts[1].parse::<f32>()) {
                                return Some((fps, lat));
                            }
                        }
                    }
                }
            }
        }
        None
    }
}
