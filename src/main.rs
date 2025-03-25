use musicbrainz_rs::entity::artist::Artist;
use musicbrainz_rs::entity::release_group::ReleaseGroup;
use musicbrainz_rs::prelude::*;
use rss::{ChannelBuilder, GuidBuilder, ItemBuilder};
use std::cmp::Reverse;
use std::io;

struct ArtistReleaseGroup {
    artist: Artist,
    release_group: ReleaseGroup,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut binding = ChannelBuilder::default();
    let channel_builder = binding
        .title("MusicBrainz Releases")
        .link("https://musicbrainz.org");

    let mut items: Vec<ArtistReleaseGroup> = Vec::new();

    for artist_mbid in io::stdin().lines() {
        let artist_mbid = match artist_mbid {
            Ok(artist_mbid) => artist_mbid,
            Err(_) => panic!("Error reading from stdin."),
        };

        let artist = match Artist::fetch().id(artist_mbid.as_str()).execute().await {
            Ok(artist) => {
                if artist.id == "" {
                    panic!("Invalid artist MBID {:?}", artist_mbid);
                }

                artist
            }
            Err(_) => panic!("Error fetching artist with MBID {:?}", artist_mbid),
        };

        let mut offset = 0;

        loop {
            let release_groups = match ReleaseGroup::browse()
                .by_artist(artist_mbid.as_str())
                // The server will not return more entities than this
                .limit(100)
                .offset(offset)
                .execute()
                .await
            {
                Ok(release_groups) => release_groups,
                Err(_) => panic!(
                    "Error fetching release groups for artist with MBID {:?}",
                    artist_mbid
                ),
            };

            for release_group in release_groups.entities.clone() {
                items.push(ArtistReleaseGroup {
                    artist: artist.clone(),
                    release_group,
                });
            }

            offset += release_groups.entities.len() as u16;
            if i32::from(offset) >= release_groups.count {
                break;
            }
        }
    }

    items.sort_by_key(|item| Reverse(item.release_group.first_release_date));

    for item in items {
        let link = format!(
            "https://musicbrainz.org/release-group/{}",
            item.release_group.id
        );

        let mut binding = ItemBuilder::default();
        let item_builder = binding
            .guid(GuidBuilder::default().value(link.clone()).build())
            .link(link)
            .title(format!(
                "{}: {}",
                item.artist.name.clone(),
                item.release_group.title
            ))
            .author(item.artist.name.clone());

        if !item.release_group.first_release_date.is_none() {
            item_builder.pub_date(Some(
                item.release_group
                    .first_release_date
                    .unwrap()
                    .format("%a, %d %b %Y 00:00:00 UT")
                    .to_string(),
            ));
        }

        channel_builder.item(item_builder.build());
    }

    match channel_builder
        .build()
        .pretty_write_to(io::stdout(), u8::try_from(' ').unwrap(), 2)
    {
        Ok(_) => Ok(()),
        Err(_) => panic!("Error writing to stdout."),
    }
}
