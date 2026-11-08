fn command_head(game: &mut crate::game::Game<'_>, command: &str) -> core::ops::ControlFlow<bool> {
    match game.toolswap_command(command) {
        Some(result) => core::ops::ControlFlow::Break(result),
        None => core::ops::ControlFlow::Continue(()),
    }
}
