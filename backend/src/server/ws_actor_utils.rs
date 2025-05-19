use actix_web_actors::ws;
use serde_json::Value;
use actix::ActorContext;

use crate::server::ws_error::ws_error_message;
use crate::server::anti_spam::AntiSpamState;

/// Trait utilitaire pour les acteurs WebSocket (matchmaking, game, etc.)
pub trait WsActorUtils {
    fn anti_spam(&mut self) -> &mut AntiSpamState;
    fn player_id(&self) -> &str;

    /// Envoie un message de ban, ferme et stoppe l'acteur.
    fn send_ban_and_close<A>(&mut self, ctx: &mut ws::WebsocketContext<A>)
    where
        A: actix::Actor<Context = ws::WebsocketContext<A>>,
    {
        let player_id = self.player_id().to_string();
        let ban_remaining_secs = self.anti_spam().ban_remaining_secs(&player_id);
        let context = serde_json::json!({
            "wallet": self.player_id(),
            "ban_remaining_secs": ban_remaining_secs
        });
        ctx.text(ws_error_message(
            "BANNED",
            "You have been banned for spamming. Please try again later.",
            Some(context),
        ));
        ctx.close(Some(ws::CloseReason {
            code: ws::CloseCode::Policy,
            description: Some("Banned for spam".into()),
        }));
        ctx.stop();
    }

    /// Envoie une erreur, applique anti-spam, et ban si besoin.
    fn send_error_and_maybe_ban<A>(
        &mut self,
        ctx: &mut ws::WebsocketContext<A>,
        code: &str,
        message: &str,
        context: Option<Value>,
    )
    where
        A: actix::Actor<Context = ws::WebsocketContext<A>>,
    {
        let player_id = self.player_id().to_string();
        if self.anti_spam().should_send_error(code, &player_id) {
            let player_id = self.player_id().to_string();
            if self.anti_spam().record_response(&player_id) {
                self.send_ban_and_close(ctx);
                return;
            }
            ctx.text(ws_error_message(code, message, context));
        }
    }

    /// Envoie une réponse JSON, ou ban si anti-spam dépassé.
    fn send_json_or_ban<A>(
        &mut self,
        ctx: &mut ws::WebsocketContext<A>,
        json_str: String,
    )
    where
        A: actix::Actor<Context = ws::WebsocketContext<A>>,
    {
        let player_id = self.player_id().to_string();
        if self.anti_spam().record_response(&player_id) {
            self.send_ban_and_close(ctx);
            return;
        }
        ctx.text(json_str);
    }
}