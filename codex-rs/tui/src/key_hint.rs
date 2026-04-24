use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyEventKind;
use crossterm::event::KeyModifiers;
use ratatui::style::Style;
use ratatui::style::Stylize;
use ratatui::text::Span;

#[cfg(test)]
const ALT_PREFIX: &str = "⌥ + ";
#[cfg(all(not(test), target_os = "macos"))]
const ALT_PREFIX: &str = "⌥ + ";
#[cfg(all(not(test), not(target_os = "macos")))]
const ALT_PREFIX: &str = "alt + ";
const CTRL_PREFIX: &str = "ctrl + ";
const SHIFT_PREFIX: &str = "shift + ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KeyBinding {
    key: KeyCode,
    modifiers: KeyModifiers,
}

impl KeyBinding {
    pub(crate) const fn new(key: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn is_press(&self, event: KeyEvent) -> bool {
        self.key == event.code
            && self.modifiers == event.modifiers
            && (event.kind == KeyEventKind::Press || event.kind == KeyEventKind::Repeat)
    }
}

pub(crate) const fn plain(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::NONE)
}

pub(crate) const fn alt(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::ALT)
}

pub(crate) const fn shift(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::SHIFT)
}

pub(crate) const fn ctrl(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::CONTROL)
}

pub(crate) const fn ctrl_alt(key: KeyCode) -> KeyBinding {
    KeyBinding::new(key, KeyModifiers::CONTROL.union(KeyModifiers::ALT))
}

fn modifiers_to_string(modifiers: KeyModifiers) -> String {
    let mut result = String::new();
    if modifiers.contains(KeyModifiers::CONTROL) {
        result.push_str(CTRL_PREFIX);
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        result.push_str(SHIFT_PREFIX);
    }
    if modifiers.contains(KeyModifiers::ALT) {
        result.push_str(ALT_PREFIX);
    }
    result
}

impl From<KeyBinding> for Span<'static> {
    fn from(binding: KeyBinding) -> Self {
        (&binding).into()
    }
}
impl From<&KeyBinding> for Span<'static> {
    fn from(binding: &KeyBinding) -> Self {
        let KeyBinding { key, modifiers } = binding;
        let modifiers = modifiers_to_string(*modifiers);
        let key = match key {
            KeyCode::Enter => "enter".to_string(),
            KeyCode::Char(' ') => "space".to_string(),
            KeyCode::Up => "↑".to_string(),
            KeyCode::Down => "↓".to_string(),
            KeyCode::Left => "←".to_string(),
            KeyCode::Right => "→".to_string(),
            KeyCode::PageUp => "pgup".to_string(),
            KeyCode::PageDown => "pgdn".to_string(),
            _ => format!("{key}").to_ascii_lowercase(),
        };
        Span::styled(format!("{modifiers}{key}"), key_hint_style())
    }
}

fn key_hint_style() -> Style {
    Style::default().dim()
}

/// Does this key event carry a modifier that suppresses normal text input?
///
/// Ctrl and Alt are the classic shortcut modifiers (Alt+Ctrl together is AltGr
/// on European layouts, which is text-producing, so we exclude it). SUPER is
/// Cmd on macOS and Win/Super on Linux — when host terminals speak the kitty
/// keyboard protocol, Cmd chords arrive here as e.g. `Char('v')` + `SUPER`
/// (`\e[118;9u`). Without this check those would be treated as plain character
/// input and the literal `v` would leak into the textarea.
pub(crate) fn has_shortcut_modifier(mods: KeyModifiers) -> bool {
    let ctrl_or_alt =
        (mods.contains(KeyModifiers::CONTROL) || mods.contains(KeyModifiers::ALT))
            && !is_altgr(mods);
    ctrl_or_alt || mods.contains(KeyModifiers::SUPER)
}

#[cfg(windows)]
#[inline]
pub(crate) fn is_altgr(mods: KeyModifiers) -> bool {
    mods.contains(KeyModifiers::ALT) && mods.contains(KeyModifiers::CONTROL)
}

#[cfg(not(windows))]
#[inline]
pub(crate) fn is_altgr(_mods: KeyModifiers) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_char_is_not_a_shortcut() {
        assert!(!has_shortcut_modifier(KeyModifiers::NONE));
        assert!(!has_shortcut_modifier(KeyModifiers::SHIFT));
    }

    #[test]
    fn ctrl_and_alt_are_shortcuts() {
        assert!(has_shortcut_modifier(KeyModifiers::CONTROL));
        assert!(has_shortcut_modifier(KeyModifiers::ALT));
    }

    #[test]
    fn super_is_a_shortcut() {
        // macOS Cmd / Linux Win arrive as SUPER when the host terminal speaks
        // the kitty keyboard protocol; without this, `Cmd+V` would leak `v`
        // into the textarea.
        assert!(has_shortcut_modifier(KeyModifiers::SUPER));
        assert!(has_shortcut_modifier(
            KeyModifiers::SUPER | KeyModifiers::SHIFT,
        ));
    }

    #[cfg(windows)]
    #[test]
    fn altgr_is_not_a_shortcut() {
        assert!(!has_shortcut_modifier(
            KeyModifiers::CONTROL | KeyModifiers::ALT,
        ));
    }
}
