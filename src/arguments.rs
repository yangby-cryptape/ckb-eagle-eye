/*  CKB Eagle Eye

    Copyright (C) 2019 Boyu Yang <yangby@cryptape.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU Affero General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Affero General Public License for more details.

    You should have received a copy of the GNU Affero General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use std::convert::TryFrom;

use property::Property;

use uckb_jsonrpc_client::url;

use crate::error::{Error, Result};

#[derive(Property)]
pub struct Arguments {
    url: url::Url,
}

pub fn build_commandline() -> Result<Arguments> {
    let yaml = clap::load_yaml!("cli.yaml");
    let matches = clap::App::from_yaml(yaml).get_matches();
    Arguments::try_from(&matches)
}

impl<'a> TryFrom<&'a clap::ArgMatches<'a>> for Arguments {
    type Error = Error;
    fn try_from(matches: &'a clap::ArgMatches) -> Result<Self> {
        let url = matches
            .value_of("url")
            .map(|url_str| url::Url::parse(url_str))
            .transpose()?
            .ok_or_else(|| Error::Unreachable("no argument 'url'".to_owned()))?;
        Ok(Self { url })
    }
}
