//! # D-Bus interface proxy for: `org.mpris.MediaPlayer2.Playlists`
//!
//! This code was generated by `zbus-xmlgen` `4.1.0` from D-Bus introspection data.
//! Source: `interface.xml`.
//!
//! You may prefer to adapt it, instead of using it verbatim.
//!
//! More information can be found in the [Writing a client proxy] section of the zbus
//! documentation.
//!
//! This type implements the [D-Bus standard interfaces], (`org.freedesktop.DBus.*`) for which the
//! following zbus API can be used:
//!
//! * [`zbus::fdo::PropertiesProxy`]
//! * [`zbus::fdo::IntrospectableProxy`]
//! * [`zbus::fdo::PeerProxy`]
//!
//! Consequently `zbus-xmlgen` did not generate code for the above interfaces.
//!
//! [Writing a client proxy]: https://dbus2.github.io/zbus/client.html
//! [D-Bus standard interfaces]: https://dbus.freedesktop.org/doc/dbus-specification.html#standard-interfaces,
use zbus::proxy;
#[proxy(interface = "org.mpris.MediaPlayer2.Playlists", assume_defaults = true)]
trait Playlists {
    /// ActivatePlaylist method
    fn activate_playlist(&self, playlist_id: &zbus::zvariant::ObjectPath<'_>) -> zbus::Result<()>;

    /// GetPlaylists method
    fn get_playlists(
        &self,
        index: u32,
        max_count: u32,
        order: &str,
        reverse_order: bool,
    ) -> zbus::Result<Vec<(zbus::zvariant::OwnedObjectPath, String, String)>>;

    /// PlaylistChanged signal
    #[zbus(signal)]
    fn playlist_changed(
        &self,
        playlist: (zbus::zvariant::ObjectPath<'_>, &str, &str),
    ) -> zbus::Result<()>;

    /// ActivePlaylist property
    #[zbus(property)]
    fn active_playlist(
        &self,
    ) -> zbus::Result<(bool, (zbus::zvariant::OwnedObjectPath, String, String))>;

    /// Orderings property
    #[zbus(property)]
    fn orderings(&self) -> zbus::Result<Vec<String>>;

    /// PlaylistCount property
    #[zbus(property)]
    fn playlist_count(&self) -> zbus::Result<u32>;
}
