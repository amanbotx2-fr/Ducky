/// Native-shell commands available during Phases 1–3.
///
/// Keep this list limited to commands backed by completed Tauri behavior.
/// Later domain phases extend it only when their authoritative Rust services
/// reach parity; placeholder commands are intentionally excluded.
pub(crate) const PHASE_1_TO_3_COMMANDS: &[&str] = &[
    "get_cursor_position",
    "move_companion_window",
    "set_companion_content_height",
    "stream_cursor_positions",
];

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::PHASE_1_TO_3_COMMANDS;

    #[test]
    fn command_manifest_contains_only_unique_phase_one_to_three_commands() {
        let unique_commands = PHASE_1_TO_3_COMMANDS
            .iter()
            .copied()
            .collect::<HashSet<_>>();

        assert_eq!(unique_commands.len(), PHASE_1_TO_3_COMMANDS.len());
        assert_eq!(
            PHASE_1_TO_3_COMMANDS,
            [
                "get_cursor_position",
                "move_companion_window",
                "set_companion_content_height",
                "stream_cursor_positions",
            ],
        );
    }
}
