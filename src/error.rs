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

use std::io;

use failure::Fail;

use uckb_jsonrpc_client::url;

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "internal error: should be unreachable, {}", _0)]
    Unreachable(String),

    #[fail(display = "io error: {}", _0)]
    IO(io::Error),
    #[fail(display = "url error: {}", _0)]
    Url(url::ParseError),
}

pub type Result<T> = ::std::result::Result<T, Error>;

macro_rules! convert_error {
    ($name:ident, $inner_error:ty) => {
        impl ::std::convert::From<$inner_error> for Error {
            fn from(error: $inner_error) -> Self {
                Self::$name(error)
            }
        }
    };
}

convert_error!(IO, io::Error);
convert_error!(Url, url::ParseError);
