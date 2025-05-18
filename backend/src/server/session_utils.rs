use actix::Addr;
use std::collections::HashMap;
use std::hash::Hash;
use std::cmp::PartialEq;

use crate::server::matchmaking::types::WalletAddress;
use crate::server::matchmaking::server::ConnectedPlayer;
use crate::server::game_session::session::GameSessionActor;
use crate::server::matchmaking::session::MatchmakingSession;

/// Fonction générique pour vérifier si une adresse correspond à celle enregistrée pour une clé donnée.
/// Retourne true si l'adresse est bien celle attendue.
pub fn is_addr_valid<K, V, A>(
    map: &HashMap<K, V>,
    key: &K,
    addr: &A,
    addr_extractor: impl Fn(&V) -> &A,
) -> bool
where
    K: Eq + Hash,
    A: PartialEq,
{
    map.get(key).map_or(false, |value| addr_extractor(value) == addr)
}

/// Fonction générique pour récupérer une valeur si l'adresse correspond à celle enregistrée.
/// Retourne None si la clé n'existe pas ou si l'adresse ne correspond pas.
pub fn get_by_addr<'a, K, V, A>(
    map: &'a HashMap<K, V>,
    key: &K,
    addr: &A,
    addr_extractor: impl Fn(&V) -> &A,
) -> Option<&'a V>
where
    K: Eq + Hash,
    A: PartialEq,
{
    map.get(key).filter(|value| addr_extractor(value) == addr)
}

/// Vérifie si l'adresse de session correspond à celle enregistrée pour ce wallet dans le matchmaking.
/// Retourne true si la session est bien celle attendue.
pub fn is_matchmaking_session_addr_valid(
    players: &HashMap<WalletAddress, ConnectedPlayer>,
    player_id: &WalletAddress,
    addr: &Addr<MatchmakingSession>,
) -> bool {
    is_addr_valid(players, player_id, addr, |player| &player.addr)
}

/// Récupère un joueur connecté si l'adresse de session correspond à celle enregistrée pour ce wallet.
/// Retourne None si le joueur n'existe pas ou si l'adresse ne correspond pas.
pub fn get_player_by_matchmaking_addr<'a>(
    players: &'a HashMap<WalletAddress, ConnectedPlayer>,
    player_id: &WalletAddress,
    addr: &Addr<MatchmakingSession>,
) -> Option<&'a ConnectedPlayer> {
    get_by_addr(players, player_id, addr, |player| &player.addr)
}

/// Vérifie si l'adresse de session correspond à celle enregistrée pour ce wallet dans la game session.
/// Retourne true si la session est bien celle attendue.
pub fn is_game_session_addr_valid(
    players: &HashMap<WalletAddress, Addr<GameSessionActor>>,
    player_id: &WalletAddress,
    addr: &Addr<GameSessionActor>,
) -> bool {
    is_addr_valid(players, player_id, addr, |a| a)
}

/// Récupère l'adresse d'une session de jeu si l'adresse fournie correspond à celle enregistrée pour ce wallet.
/// Retourne None si le joueur n'existe pas ou si l'adresse ne correspond pas.
pub fn get_player_by_game_session_addr<'a>(
    players: &'a HashMap<WalletAddress, Addr<GameSessionActor>>,
    player_id: &WalletAddress,
    addr: &Addr<GameSessionActor>,
) -> Option<&'a Addr<GameSessionActor>> {
    get_by_addr(players, player_id, addr, |a| a)
}

/// Vérifie si l'adresse de session correspond à celle enregistrée pour ce wallet dans la liste des spectateurs.
/// Retourne true si la session est bien celle attendue.
pub fn is_game_session_spectator_addr_valid(
    spectators: &HashMap<WalletAddress, Addr<GameSessionActor>>,
    wallet: &WalletAddress,
    addr: &Addr<GameSessionActor>,
) -> bool {
    is_addr_valid(spectators, wallet, addr, |a| a)
}
