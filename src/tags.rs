pub struct Tags {
    native_mpd: Vec<(String, String)>,
    raw_comments: Vec<(String, String)>,
}

impl Tags {
    pub fn from_song(song: &mpd::Song) -> Tags {
        Tags {
            native_mpd: song.tags.clone(),
            raw_comments: vec![],
        }
    }

    pub fn from_song_and_raw_comments(
        song: &mpd::Song,
        raw_comments: Vec<(String, String)>,
    ) -> Tags {
        Tags {
            native_mpd: song.tags.clone(),
            raw_comments,
        }
    }

    pub fn get<'a>(&'a self, tag: &'a str) -> Vec<&'a str> {
        let mut tags = tag_filter(&*self.native_mpd, tag).collect::<Vec<_>>();
        if tags.is_empty() {
            tags.extend(tag_filter(&*self.raw_comments, tag));
        }
        tags.sort_unstable();
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
