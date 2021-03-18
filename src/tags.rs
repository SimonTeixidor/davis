pub struct Tags {
    native_mpd: Vec<(String, String)>,
    raw_comments: Vec<(String, String)>,
}

impl Tags {
    pub fn from_song(
        song: &mpd::Song,
        client: &mut mpd::Client,
    ) -> Result<Tags, mpd::error::Error> {
        let raw_comments = client.readcomments(&song)?.flatten().collect::<Vec<_>>();
        Ok(Tags {
            native_mpd: song.tags.clone(),
            raw_comments,
        })
    }

    pub fn get<'a>(&'a self, tag: &'a str) -> Vec<&'a str> {
        let mut tags = tag_filter(&*self.native_mpd, tag)
            .chain(tag_filter(&*self.raw_comments, tag))
            .collect::<Vec<_>>();
        tags.dedup();
        tags
    }

    pub fn get_option<'a>(&'a self, tag: &'a str) -> Option<Vec<&'a str>> {
        let vals = self.get(tag);
        if vals.is_empty() {
            None
        } else {
            Some(vals)
        }
    }

    pub fn get_option_joined<'a>(&'a self, tag: &'a str) -> Option<String> {
        let vals = self.get(tag);
        if vals.is_empty() {
            None
        } else {
            Some(vals.join(", "))
        }
    }
}

fn tag_filter<'a>(vals: &'a [(String, String)], tag: &'a str) -> impl Iterator<Item = &'a str> {
    vals.iter().filter_map(move |(k, v)| {
        if k.to_uppercase() == tag.to_uppercase() {
            Some(&**v)
        } else {
            None
        }
    })
}
