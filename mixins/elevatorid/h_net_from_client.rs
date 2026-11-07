fn elevator_net_from_client(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    crate::game::elevatorid::from_client(player, channel, data);
}
