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

use std::{collections::VecDeque, fmt};

use property::Property;

use uckb_jsonrpc_client::{
    client::CkbSyncClient,
    interfaces::types::{packed, prelude::*, rpc},
};

use crate::{arguments, error};

#[derive(Property, Default, Debug, Clone, Copy)]
pub struct Dao {
    total: u64,
    rate: u64,
    secondary: u64,
    occupied: u64,
}

#[derive(Property, Default, Debug, Clone, Copy)]
pub struct Cellbase {
    total: u64,
    primary: u64,
    secondary: u64,
    tx_fee: u64,
    proposal_reward: u64,
}

#[derive(Property)]
pub struct Summary {
    client: CkbSyncClient,

    next_block: u64,

    total: u64,
    total_primary: u64,
    total_secondary: u64,
    total_miner_secondary: u64,
    total_tx_fee: u64,
    total_proposal_reward: u64,

    epoch_primary: u64,
    epoch_secondary: u64,

    genesis_cellbase: u64,
    primary_burned: u64,
    epoch_primary_expected: u64,
    epoch_secondary_expected: u64,

    daos: VecDeque<Dao>,
    dao_prev: Dao,
}

impl fmt::Display for Dao {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "t : {:20}; r : {:20}; s : {:20}; o : {:20}",
            self.total, self.rate, self.secondary, self.occupied
        )
    }
}

impl fmt::Display for Cellbase {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "t : {:20}; p : {:20}; s : {:20}; tx: {:20}; pr: {:20}",
            self.total, self.primary, self.secondary, self.tx_fee, self.proposal_reward
        )
    }
}

impl From<packed::Byte32> for Dao {
    fn from(dao: packed::Byte32) -> Self {
        let slice = dao.raw_data();
        let mut tmp = [0u8; 8];
        tmp.copy_from_slice(&slice[0..8]);
        let total = u64::from_le_bytes(tmp);
        tmp.copy_from_slice(&slice[8..16]);
        let rate = u64::from_le_bytes(tmp);
        tmp.copy_from_slice(&slice[16..24]);
        let secondary = u64::from_le_bytes(tmp);
        tmp.copy_from_slice(&slice[24..32]);
        let occupied = u64::from_le_bytes(tmp);
        Self {
            total,
            rate,
            secondary,
            occupied,
        }
    }
}

impl From<rpc::BlockReward> for Cellbase {
    fn from(reward: rpc::BlockReward) -> Self {
        let total: u64 = reward.total.into();
        let primary: u64 = reward.primary.into();
        let secondary: u64 = reward.secondary.into();
        let tx_fee: u64 = reward.tx_fee.into();
        let proposal_reward: u64 = reward.proposal_reward.into();
        Self {
            total,
            primary,
            secondary,
            tx_fee,
            proposal_reward,
        }
    }
}

impl Dao {
    pub fn trace(self) {
        log::trace!("            Dao:");
        log::trace!("                t : {:20}", self.total);
        log::trace!("                r : {:20}", self.rate);
        log::trace!("                s : {:20}", self.secondary);
        log::trace!("                o : {:20}", self.occupied);
    }
}

impl Cellbase {
    pub fn trace(self) {
        log::trace!("            Cellbase:");
        log::trace!("                t : {:20}", self.total);
        log::trace!("                p : {:20}", self.primary);
        log::trace!("                s : {:20}", self.secondary);
        log::trace!("                tx: {:20}", self.tx_fee);
        log::trace!("                pr: {:20}", self.proposal_reward);
    }

    pub fn check_total(self) -> bool {
        self.total == self.primary + self.secondary + self.tx_fee + self.proposal_reward
    }
}

impl Summary {
    pub fn new(client: CkbSyncClient) -> Self {
        let next_block = 0;
        Self {
            client,
            next_block,
            total: 0,
            total_primary: 0,
            total_secondary: 0,
            total_miner_secondary: 0,
            total_tx_fee: 0,
            total_proposal_reward: 0,
            epoch_primary: 0,
            epoch_secondary: 0,
            genesis_cellbase: 0,
            primary_burned: 0,
            epoch_primary_expected: 0,
            epoch_secondary_expected: 0,
            daos: VecDeque::new(),
            dao_prev: Dao::default(),
        }
    }

    pub fn add_cellbase(&mut self, cellbase: Cellbase) {
        self.total += cellbase.total;
        self.total_primary += cellbase.primary;
        self.total_miner_secondary += cellbase.secondary;
        self.total_tx_fee += cellbase.tx_fee;
        self.total_proposal_reward += cellbase.proposal_reward;
        self.epoch_primary += cellbase.primary;
    }

    pub fn total_issuance(&self) -> u64 {
        self.genesis_cellbase + self.total_primary + self.total_secondary + self.primary_burned
    }

    pub fn next(&mut self) {
        let block_number = self.next_block();
        self.next_block += 1;
        let block = self.client().block_by_number(block_number).unwrap();
        if block.epoch().index() % 100 == 0 {
            log::info!("    check {:#} -- {}", block.epoch(), block.number());
        } else {
            log::debug!("    check {:#} -- {}", block.epoch(), block.number());
        }
        let tx0_outputs = block
            .data()
            .transactions()
            .get_unchecked(0)
            .raw()
            .outputs()
            .into_iter()
            .map(|output| Unpack::<u64>::unpack(&output.capacity()))
            .sum::<u64>();
        let dao_curr: Dao = block.dao().into();
        self.daos.push_back(dao_curr);
        if block_number == 0 {
            self.genesis_cellbase = tx0_outputs;
            self.primary_burned = dao_curr.total() - tx0_outputs - dao_curr.secondary();
            self.epoch_secondary = dao_curr.secondary();
            self.total_secondary = dao_curr.secondary();
        } else if block_number < 11 {
        } else if block_number == 11 {
            self.dao_prev = self.daos.pop_front().unwrap();
            self.dao_prev().trace();
        } else {
            if block.epoch().index() == 11 {
                let epoch_number = block.epoch().number() - 1;
                if epoch_number == 0 {
                    self.epoch_primary_expected = self.epoch_primary + self.primary_burned;
                    self.epoch_secondary_expected = self.epoch_secondary;
                } else {
                    assert_eq!(self.epoch_primary, self.epoch_primary_expected);
                    assert_eq!(self.epoch_secondary, self.epoch_secondary_expected);
                }
                self.epoch_primary = 0;
                self.epoch_secondary = 0;
            }
            let dao = self.daos.pop_front().unwrap();
            dao.trace();
            log::trace!("            Tx(0) : {:20}", tx0_outputs);
            let details = self
                .client()
                .get_cellbase_output_capacity_details(block.hash().unpack())
                .unwrap();
            let cellbase: Cellbase = details.into();
            cellbase.trace();
            let block_secondary = dao.total() - self.dao_prev().total() - cellbase.primary();
            log::trace!("            Sec   : {:20}", block_secondary);
            self.epoch_secondary += block_secondary;
            self.total_secondary += block_secondary;
            assert!(cellbase.check_total());
            assert_eq!(tx0_outputs, cellbase.total());
            self.add_cellbase(cellbase);
            assert_eq!(dao.total(), self.total_issuance());
            let miner_secondary = (u128::from(block_secondary)
                * u128::from(self.dao_prev().occupied()))
                / u128::from(self.dao_prev().total());
            assert_eq!(miner_secondary as u64, cellbase.secondary());
            self.dao_prev = dao;
        }
    }
}

pub fn inspect(args: &arguments::Arguments) -> error::Result<()> {
    let client = CkbSyncClient::new(args.url().to_owned());

    let tip_number = client.tip_number().unwrap();
    log::info!("current block number is {}", tip_number);

    let mut summary = Summary::new(client);
    for _ in 0..=tip_number {
        summary.next();
    }

    log::info!("DONE");

    Ok(())
}
