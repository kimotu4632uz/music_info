pub mod oauth2;

pub mod music_brainz;
pub use music_brainz::MusicBrainz;

pub mod spotify;
pub use spotify::Spotify;

pub type Client = ureq::Agent;

#[inline]
pub fn http_client() -> Client {
    ureq::agent()
}
