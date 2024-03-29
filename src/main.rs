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

use std::{env, process, str};

mod arguments;
mod error;
mod issuance;

fn main() {
    {
        let log_key = "CKB_EAGLE_EYE_LOG";
        if env::var(log_key).is_err() {
            let pkgname = env!("CARGO_PKG_NAME");
            let log_value = format!("error,{}=trace", str::replace(pkgname, "-", "_"));
            env::set_var(log_key, log_value);
        }
        pretty_env_logger::try_init_timed_custom_env(log_key).unwrap();
    }
    log::info!("Begin to inspect a CKB chain.");

    if let Err(error) = execute() {
        eprintln!("fatal: {}", error);
        process::exit(1);
    }
}

fn execute() -> error::Result<()> {
    let args = arguments::build_commandline()?;
    issuance::inspect(&args)
}
