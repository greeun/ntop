use ntop::config::Config;
use std::time::Duration;

#[test]
fn test_default_config() {
    let config = Config::default();

    // GeneralConfig defaults
    assert_eq!(config.general.refresh_interval, 1);
    assert_eq!(config.general.default_signal, "SIGTERM");
    assert_eq!(config.general.graceful_timeout, 10);
    assert_eq!(config.general.confirm_before_kill, true);

    // DisplayConfig defaults
    assert_eq!(config.display.show_tree, true);
    assert_eq!(config.display.color_theme, "auto");
    assert_eq!(config.display.mask_env_values, true);

    // FilterConfig defaults
    assert_eq!(config.filter.include_bun, false);
    assert_eq!(config.filter.include_tsx, false);
    assert_eq!(config.filter.include_ts_node, false);
}

#[test]
fn test_parse_toml_config() {
    let toml_str = r#"
[general]
refresh_interval = 5
default_signal = "SIGKILL"
graceful_timeout = 30
confirm_before_kill = false

[display]
show_tree = false
color_theme = "dark"
mask_env_values = false

[filter]
include_bun = true
include_tsx = true
include_ts_node = true
"#;

    let config: Config = toml::from_str(toml_str).unwrap();

    assert_eq!(config.general.refresh_interval, 5);
    assert_eq!(config.general.default_signal, "SIGKILL");
    assert_eq!(config.general.graceful_timeout, 30);
    assert_eq!(config.general.confirm_before_kill, false);

    assert_eq!(config.display.show_tree, false);
    assert_eq!(config.display.color_theme, "dark");
    assert_eq!(config.display.mask_env_values, false);

    assert_eq!(config.filter.include_bun, true);
    assert_eq!(config.filter.include_tsx, true);
    assert_eq!(config.filter.include_ts_node, true);
}

#[test]
fn test_partial_toml_uses_defaults() {
    let toml_str = r#"
[general]
refresh_interval = 3
"#;

    let config: Config = toml::from_str(toml_str).unwrap();

    // Explicitly set value
    assert_eq!(config.general.refresh_interval, 3);

    // Remaining general defaults
    assert_eq!(config.general.default_signal, "SIGTERM");
    assert_eq!(config.general.graceful_timeout, 10);
    assert_eq!(config.general.confirm_before_kill, true);

    // Display defaults (entire section missing)
    assert_eq!(config.display.show_tree, true);
    assert_eq!(config.display.color_theme, "auto");
    assert_eq!(config.display.mask_env_values, true);

    // Filter defaults (entire section missing)
    assert_eq!(config.filter.include_bun, false);
    assert_eq!(config.filter.include_tsx, false);
    assert_eq!(config.filter.include_ts_node, false);
}

#[test]
fn test_config_refresh_duration() {
    let mut config = Config::default();
    assert_eq!(config.refresh_duration(), Duration::from_secs(1));

    config.general.refresh_interval = 5;
    assert_eq!(config.refresh_duration(), Duration::from_secs(5));
}

#[test]
fn test_config_graceful_duration() {
    let mut config = Config::default();
    assert_eq!(config.graceful_duration(), Duration::from_secs(10));

    config.general.graceful_timeout = 30;
    assert_eq!(config.graceful_duration(), Duration::from_secs(30));
}
