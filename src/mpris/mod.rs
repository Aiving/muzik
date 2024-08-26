use generated::player::{Metadata, PlaybackStatus, PlayerProxy};
use zbus::{proxy::CacheProperties, Connection};

pub mod generated;

#[derive(Debug)]
pub struct Player<'a> {
    player_proxy: PlayerProxy<'a>,
}

impl<'a> Player<'a> {
    pub async fn all() -> Vec<Self> {
        let connection = Connection::session().await.unwrap();

        let message = connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/",
                Some("org.freedesktop.DBus"),
                "ListNames",
                &(),
            )
            .await
            .unwrap();

        let names: Vec<String> = message.body().deserialize().unwrap();
        let mut names = names
            .into_iter()
            .filter(|name| name.starts_with("org.mpris.MediaPlayer2"))
            .collect::<Vec<_>>();

        names.sort_by_key(|name| name.to_lowercase());

        let mut players = vec![];

        for name in names {
            let player_proxy = PlayerProxy::builder(&connection)
                .destination(name)
                .and_then(|builder| builder.path("/org/mpris/MediaPlayer2"))
                .map(|builder| {
                    builder
                        .cache_properties(CacheProperties::Yes)
                        .uncached_properties(&["Position"])
                        .build()
                });

            if let Ok(player_proxy) = player_proxy {
                if let Ok(player_proxy) = player_proxy.await {
                    players.push(Self { player_proxy });
                }
            }
        }

        players
    }

    pub async fn find_active() -> Option<Self> {
        let players = Self::all().await;

        if players.is_empty() {
            return None;
        }

        let mut first_paused: Option<Player> = None;
        let mut first_with_track: Option<Player> = None;
        let mut first_found: Option<Player> = None;

        for player in players {
            let player_status = player.get_playback_status().await?;

            if player_status == PlaybackStatus::Playing {
                return Some(player);
            }

            if first_paused.is_none() && player_status == PlaybackStatus::Paused {
                first_paused.replace(player);
            } else if first_with_track.is_none() && !player.get_metadata().await?.is_empty() {
                first_with_track.replace(player);
            } else if first_found.is_none() {
                first_found.replace(player);
            }
        }

        first_paused.or(first_with_track).or(first_found)
    }

    pub async fn get_metadata(&self) -> Option<Metadata> {
        self.player_proxy.metadata().await.ok()
    }

    pub async fn get_playback_status(&self) -> Option<PlaybackStatus> {
        self.player_proxy.playback_status().await.ok()
    }

    pub async fn get_position(&self) -> Option<i64> {
        self.player_proxy.position().await.ok()
    }
}
