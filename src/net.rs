pub mod oauth2;
pub mod music_brainz;
pub mod spotify;

pub type Client = ureq::Agent;

#[inline]
pub fn http_client() -> Client {
    ureq::agent()
}
