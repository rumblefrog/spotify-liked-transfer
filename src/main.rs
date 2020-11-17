use std::env::var;

use rspotify::client::Spotify;
use rspotify::oauth2::{SpotifyClientCredentials, SpotifyOAuth};
use rspotify::util::{process_token, request_token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut oauth = SpotifyOAuth::default()
        .scope("user-library-read playlist-modify-public")
        .redirect_uri("http://localhost/")
        .build();

    request_token(&mut oauth);

    println!("Copy and paste the redirected URL");

    let mut url = String::new();

    std::io::stdin().read_line(&mut url)?;

    if let Some(token) = process_token(&mut oauth, &mut url).await {
        // rspotify utilizes dotenv
        let client_credential = SpotifyClientCredentials::default().build();

        let user_id = &var("USER_ID")?;

        let spotify = Spotify::default()
            .client_credentials_manager(client_credential)
            .access_token(&token.access_token)
            .build();

        let playlist = spotify
            .user_playlist_create(user_id, "Liked", true, None)
            .await?;

        let tracks = spotify.current_user_saved_tracks(50, 0).await?;

        spotify.user_playlist_add_tracks(
            user_id,
            &playlist.id,
            &tracks
                .items
                .iter()
                .map(|i| i.track.id.as_ref().unwrap().to_string())
                .collect::<Vec<String>>(),
            None,
        ).await?;

        // Uncertain about how aggressive the ratelimit is
        // For my purpose, it paginates 14 times which is under the ratelimit
        let pages = (tracks.total as f32 / 50.0).ceil() as u32;

        for i in 1..pages {
            let track_offset = spotify.current_user_saved_tracks(50, i * 50).await?;

            spotify.user_playlist_add_tracks(
                user_id,
                &playlist.id,
                &track_offset
                    .items
                    .iter()
                    .map(|i| i.track.id.as_ref().unwrap().to_string())
                    .collect::<Vec<String>>(),
                None,
            ).await?;
        }
    }

    Ok(())
}
