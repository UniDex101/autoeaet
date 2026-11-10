fn auto_eat_tick(game: &mut crate::game::Game<'_>) {
    crate::game::auto_eat_baritone::on_tick(game);
}
