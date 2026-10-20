use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Slide {
    pub title: String,
    pub lines: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Set {
    pub title: String,
    pub slides: Vec<Slide>,
}

impl Set {
    pub fn demo() -> Self {
        Self {
            title: "Songverse Offline Demo".into(),
            slides: vec![
                Slide { title: "Welcome".into(), lines: vec!["Songverse Player".into(), "Rust + Chromium + HDMI".into()] },
                Slide { title: "Verse".into(), lines: vec!["Offline-ready screen prototype".into()] },
                Slide { title: "Chorus".into(), lines: vec!["Next: pairing, caching and audio".into()] },
            ],
        }
    }
}

pub fn advance(current: usize, count: usize, delta: i32) -> usize {
    if count == 0 { return 0; }
    (current as i64 + delta as i64).clamp(0, (count - 1) as i64) as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn navigation_is_bounded() {
        assert_eq!(advance(0, 3, -1), 0);
        assert_eq!(advance(0, 3, 1), 1);
        assert_eq!(advance(2, 3, 1), 2);
        assert_eq!(advance(0, 0, 1), 0);
    }
}
