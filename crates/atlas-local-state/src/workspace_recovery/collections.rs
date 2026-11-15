use super::*;
use crate::SavedListItem;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct List {
    key: String,
    slug: String,
    name: String,
    description: Option<String>,
    revision: i64,
    created_at: String,
    updated_at: String,
    tags: Vec<Tag>,
    items: Vec<SavedListItem>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Tag {
    name: String,
    created_at: String,
}

impl List {
    pub(super) fn capture(db: &Connection, owner: i64) -> LocalStateResult<Vec<Self>> {
        let mut roots = rows(
            db,
            "SELECT id,list_key,slug,name,description,revision,created_at,updated_at
             FROM saved_lists WHERE owner_id=?1 ORDER BY list_key",
            [owner],
            |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    Self {
                        key: row.get(1)?,
                        slug: row.get(2)?,
                        name: row.get(3)?,
                        description: row.get(4)?,
                        revision: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                        tags: vec![],
                        items: vec![],
                    },
                ))
            },
        )?;
        for (id, list) in &mut roots {
            list.tags = rows(
                db,
                "SELECT tag,created_at FROM saved_list_tags WHERE list_id=?1 ORDER BY tag",
                [*id],
                |row| {
                    Ok(Tag {
                        name: row.get(0)?,
                        created_at: row.get(1)?,
                    })
                },
            )?;
            list.items = rows(db,
                "SELECT record_key,position,note,record_title_snapshot,record_kind_snapshot,
                        added_at,updated_at FROM saved_list_items WHERE list_id=?1 ORDER BY position",
                [*id], |row| Ok(SavedListItem {
                    record_key: row.get(0)?, position: row.get(1)?, note: row.get(2)?,
                    record_title_snapshot: row.get(3)?, record_kind_snapshot: row.get(4)?,
                    added_at: row.get(5)?, updated_at: row.get(6)?,
                }))?;
        }
        Ok(roots.into_iter().map(|(_, list)| list).collect())
    }

    pub(super) fn validate(
        &self,
        keys: &mut BTreeSet<String>,
        slugs: &mut BTreeSet<String>,
    ) -> LocalStateResult<()> {
        crate::slug::validate_slug(&self.slug)?;
        unique(keys, &self.key, "list key")?;
        unique(slugs, &self.slug, "list slug")?;
        require(self.revision >= 0, "invalid list revision")?;
        let mut tags = BTreeSet::new();
        for tag in &self.tags {
            unique(&mut tags, &tag.name, "list tag")?;
        }
        let mut records = BTreeSet::new();
        let mut positions = BTreeSet::new();
        for item in &self.items {
            unique(&mut records, &item.record_key, "list record key")?;
            require(positions.insert(item.position), "duplicate list position")?;
        }
        Ok(())
    }

    pub(super) fn record_keys(&self) -> impl Iterator<Item = &str> {
        self.items.iter().map(|item| item.record_key.as_str())
    }

    pub(super) fn restore(&self, db: &Connection, owner: i64) -> LocalStateResult<()> {
        db.execute(
            "INSERT INTO saved_lists(owner_id,list_key,slug,name,description,revision,created_at,updated_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![owner,self.key,self.slug,self.name,self.description,self.revision,self.created_at,self.updated_at],
        )?;
        let id = db.last_insert_rowid();
        for tag in &self.tags {
            db.execute(
                "INSERT INTO saved_list_tags(list_id,tag,created_at) VALUES(?1,?2,?3)",
                params![id, tag.name, tag.created_at],
            )?;
        }
        for item in &self.items {
            db.execute(
                "INSERT INTO saved_list_items(list_id,record_key,position,note,record_title_snapshot,
                                              record_kind_snapshot,added_at,updated_at)
                 VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![id,item.record_key,item.position,item.note,item.record_title_snapshot,
                        item.record_kind_snapshot,item.added_at,item.updated_at],
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct Party {
    key: String,
    name: String,
    revision: i64,
    members: Vec<Member>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Member {
    key: String,
    name: String,
    level: Option<i64>,
    position: i64,
}

impl Party {
    pub(super) fn capture(db: &Connection, owner: i64) -> LocalStateResult<Vec<Self>> {
        let mut parties = rows(
            db,
            "SELECT party_key,name,revision FROM parties WHERE owner_id=?1 ORDER BY party_key",
            [owner],
            |row| {
                Ok(Self {
                    key: row.get(0)?,
                    name: row.get(1)?,
                    revision: row.get(2)?,
                    members: vec![],
                })
            },
        )?;
        for party in &mut parties {
            party.members = rows(
                db,
                "SELECT member_key,name,level,position FROM party_members WHERE party_key=?1 ORDER BY position",
                [&party.key],
                |row| {
                    Ok(Member {
                        key: row.get(0)?,
                        name: row.get(1)?,
                        level: row.get(2)?,
                        position: row.get(3)?,
                    })
                },
            )?;
        }
        Ok(parties)
    }

    pub(super) fn validate(
        &self,
        parties: &mut BTreeSet<String>,
        members: &mut BTreeSet<String>,
    ) -> LocalStateResult<()> {
        let valid_name = |name: &str| {
            (1..=255).contains(
                &name
                    .trim_matches(' ')
                    .split('\0')
                    .next()
                    .unwrap_or_default()
                    .chars()
                    .count(),
            )
        };
        unique(parties, &self.key, "party key")?;
        require(
            self.revision >= 0 && valid_name(&self.name),
            "invalid party",
        )?;
        let mut positions = BTreeSet::new();
        for member in &self.members {
            unique(members, &member.key, "party member key")?;
            require(
                valid_name(&member.name)
                    && member.position >= 0
                    && positions.insert(member.position)
                    && member.level.is_none_or(|level| (1..=20).contains(&level)),
                "invalid party member",
            )?;
        }
        Ok(())
    }

    pub(super) fn restore(&self, db: &Connection, owner: i64) -> LocalStateResult<()> {
        db.execute(
            "INSERT INTO parties(party_key,owner_id,name,revision) VALUES(?1,?2,?3,?4)",
            params![self.key, owner, self.name, self.revision],
        )?;
        for member in &self.members {
            db.execute("INSERT INTO party_members(member_key,party_key,name,level,position) VALUES(?1,?2,?3,?4,?5)",
                params![member.key,self.key,member.name,member.level,member.position])?;
        }
        Ok(())
    }
}
