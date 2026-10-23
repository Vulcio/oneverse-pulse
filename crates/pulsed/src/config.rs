use std::ffi::OsString;

#[derive(Debug, PartialEq)]
pub enum Command {
    Run { dev: bool },
    Help,
    Version,
}

pub fn command(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut dev = false;
    for arg in args {
        match arg.as_str() {
            "--dev" => dev = true,
            "--help" | "-h" => return Ok(Command::Help),
            "--version" | "-V" => return Ok(Command::Version),
            _ => return Err(format!("unknown argument: {arg}; use pulsed --help")),
        }
    }
    Ok(Command::Run { dev })
}

fn preferred(current: Option<OsString>, legacy: Option<OsString>) -> Option<OsString> {
    current.or(legacy)
}

pub fn bundle_dir() -> Option<OsString> {
    let current = std::env::var_os("PULSE_BUNDLE_DIR");
    let legacy = std::env::var_os("SONGVERSE_BUNDLE_DIR");
    if legacy.is_some() {
        eprintln!("Warning: SONGVERSE_BUNDLE_DIR is deprecated; use PULSE_BUNDLE_DIR. Legacy variables are supported for one release only.");
    }
    preferred(current, legacy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_mode_requires_an_explicit_flag() {
        assert_eq!(command([]), Ok(Command::Run { dev: false }));
        assert_eq!(command(["--dev".into()]), Ok(Command::Run { dev: true }));
        assert!(command(["--deev".into()]).is_err());
        assert_eq!(command(["--help".into()]), Ok(Command::Help));
        assert_eq!(command(["--version".into()]), Ok(Command::Version));
    }

    #[test]
    fn current_configuration_wins_including_an_explicit_empty_value() {
        let old = Some(OsString::from("old"));
        assert_eq!(
            preferred(Some("new".into()), old.clone()),
            Some("new".into())
        );
        assert_eq!(preferred(None, old), Some("old".into()));
        assert_eq!(
            preferred(Some("".into()), Some("old".into())),
            Some("".into())
        );
        assert_eq!(preferred(None, None), None);
    }
}
