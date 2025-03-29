use musicbrainz_rs::entity::release_group::{
    ReleaseGroup, ReleaseGroupPrimaryType, ReleaseGroupSecondaryType,
};
use musicbrainz_rs::prelude::*;
use rss::{ChannelBuilder, GuidBuilder, ItemBuilder};
use std::cmp::Reverse;
use std::collections::HashMap;
use std::io;

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut binding = ChannelBuilder::default();
    let channel_builder = binding
        .title("MusicBrainz Releases")
        .link("https://musicbrainz.org");

    let mut keyed_items: HashMap<String, ReleaseGroup> = HashMap::new();

    for artist_mbid in io::stdin().lines() {
        let artist_mbid = match artist_mbid {
            Ok(artist_mbid) => artist_mbid,
            Err(_) => panic!("Error reading from stdin."),
        };

        let mut offset = 0;

        loop {
            let release_groups = match ReleaseGroup::browse()
                .by_artist(artist_mbid.as_str())
                .with_artist_credits()
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
                keyed_items.insert(release_group.clone().id, release_group);
            }

            offset += release_groups.entities.len() as u16;
            if i32::from(offset) >= release_groups.count {
                break;
            }
        }
    }

    let mut items = keyed_items.values().collect::<Vec<&ReleaseGroup>>();
    items.sort_by_key(|item| Reverse(item.first_release_date));

    for item in items {
        let link = format!("https://musicbrainz.org/release-group/{}", item.id);

        let mut author = "".to_owned();
        for artist_credit in item.artist_credit.clone().unwrap() {
            author.push_str(artist_credit.name.as_str());
            match artist_credit.joinphrase {
                None => {}
                Some(join_phrase) => {
                    author.push_str(join_phrase.as_str());
                }
            }
        }

        let mut types = Vec::new();
        match item.primary_type.clone() {
            None => {}
            Some(primary_type) => match primary_type {
                ReleaseGroupPrimaryType::Album => types.push("Album"),
                ReleaseGroupPrimaryType::Single => types.push("Single"),
                ReleaseGroupPrimaryType::Ep => types.push("EP"),
                ReleaseGroupPrimaryType::Broadcast => types.push("Broadcast"),
                _ => {}
            },
        }
        for secondary_type in item.secondary_types.clone() {
            match secondary_type {
                ReleaseGroupSecondaryType::AudioDrama => types.push("Audio drama"),
                ReleaseGroupSecondaryType::Audiobook => types.push("Audiobook"),
                ReleaseGroupSecondaryType::Compilation => types.push("Compilation"),
                ReleaseGroupSecondaryType::DjMix => types.push("DJ-mix"),
                ReleaseGroupSecondaryType::Demo => types.push("Demo"),
                ReleaseGroupSecondaryType::Interview => types.push("Interview"),
                ReleaseGroupSecondaryType::Live => types.push("Live"),
                ReleaseGroupSecondaryType::MixtapeStreet => types.push("Mixtape/Street"),
                ReleaseGroupSecondaryType::Remix => types.push("Remix"),
                ReleaseGroupSecondaryType::Soundtrack => types.push("Soundtrack"),
                ReleaseGroupSecondaryType::Spokenword => types.push("Spokenword"),
                _ => {}
            }
        }

        let mut title = format!("{}: {}", author, item.title);
        if types.len() > 0 {
            title = format!("{} ({})", title, types.join("; "))
        }

        let mut binding = ItemBuilder::default();
        let item_builder = binding
            .guid(GuidBuilder::default().value(link.clone()).build())
            .link(link)
            .title(title)
            .author(author);

        match item.first_release_date {
            None => {}
            Some(first_release_date) => {
                item_builder.pub_date(
                    first_release_date
                        .format("%a, %d %b %Y 00:00:00 UT")
                        .to_string(),
                );
            }
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
