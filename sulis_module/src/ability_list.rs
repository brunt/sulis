//  This file is part of Sulis, a turn based RPG written in Rust.
//  Copyright 2018 Jared Stephen
//
//  Sulis is free software: you can redistribute it and/or modify
//  it under the terms of the GNU General Public License as published by
//  the Free Software Foundation, either version 3 of the License, or
//  (at your option) any later version.
//
//  Sulis is distributed in the hope that it will be useful,
//  but WITHOUT ANY WARRANTY; without even the implied warranty of
//  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//  GNU General Public License for more details.
//
//  You should have received a copy of the GNU General Public License
//  along with Sulis.  If not, see <http://www.gnu.org/licenses/>

use std::io::Error;
use std::rc::Rc;
use std::slice::Iter;

use serde::Deserialize;

use sulis_core::util::unable_to_create_error;

use crate::{Ability, Module};

#[derive(Debug)]
pub struct Entry {
    pub ability: Rc<Ability>,
    pub position: (f32, f32),
}

#[derive(Debug)]
pub struct AbilityList {
    pub id: String,
    pub name: String,
    entries: Vec<Entry>,
}

impl AbilityList {
    pub fn new(builder: AbilityListBuilder, module: &Module) -> Result<AbilityList, Error> {
        let entries = builder
            .abilities
            .into_iter()
            .map(|entry| {
                module
                    .abilities
                    .get(&entry.id)
                    .map(|ability| Entry {
                        ability: Rc::clone(ability),
                        position: entry.position,
                    })
                    .ok_or_else(|| {
                        warn!("Unable to find ability '{}'", entry.id);
                        unable_to_create_error("ability_list", &builder.id)
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AbilityList {
            id: builder.id,
            name: builder.name,
            entries,
        })
    }

    pub fn iter(&self) -> Iter<Entry> {
        self.entries.iter()
    }
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct EntryBuilder {
    id: String,
    position: (f32, f32),
}

#[derive(Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct AbilityListBuilder {
    pub id: String,
    pub name: String,
    abilities: Vec<EntryBuilder>,
}
