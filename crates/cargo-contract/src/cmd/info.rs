// Copyright 2018-2023 Parity Technologies (UK) Ltd.
// This file is part of cargo-contract.
//
// cargo-contract is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// cargo-contract is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with cargo-contract.  If not, see <http://www.gnu.org/licenses/>.

use super::{
    runtime_api::api::{self},
    Client, DefaultConfig,
};
use crate::{
    cmd::{
        runtime_api::api::runtime_types::pallet_contracts::storage::ContractInfo,
        ErrorVariant,
    },
    name_value_println,
};
use anyhow::{anyhow, Result};
use std::fmt::Debug;
use subxt::{Config, OnlineClient};

#[derive(Debug, clap::Args)]
#[clap(name = "info", about = "Get infos from a contract")]
pub struct InfoCommand {
    /// The address of the contract to display info of.
    #[clap(name = "contract", long, env = "CONTRACT")]
    contract: <DefaultConfig as Config>::AccountId,
    #[clap(
        name = "url",
        long,
        value_parser,
        default_value = "ws://localhost:9944"
    )]
    url: url::Url,
    /// Export the call output as JSON.
    #[clap(name = "output-json", long)]
    output_json: bool,
}

impl InfoCommand {
    pub fn run(&self) -> Result<(), ErrorVariant> {
        tracing::debug!(
            "Getting contract information for AccountId {:?}",
            self.contract
        );

        async_std::task::block_on(async {
            let url = self.url.clone();
            let client = OnlineClient::<DefaultConfig>::from_url(url).await?;

            let info_result = self.fetch_contract_info(&client).await?;

            match info_result {
                Some(info_result) => {
                    // InfoCommand::basic_display_format_contract_info(info_result);
                    let output_type = match self.output_json {
                        true => OutputType::Json,
                        false => OutputType::HumanReadable,
                    };
                    if matches!(output_type, OutputType::Json) {
                        InfoCommand::basic_display_format_contract_info(info_result);
                    } else {
                        InfoCommand::serialize_json(info_result);
                    }
                    Result::<(), ErrorVariant>::Ok(())
                }
                None => {
                    return Err(anyhow!(
                        "No contract information was found for account id {}",
                        self.contract
                    )
                    .into())
                }
            }
        })
    }

    async fn fetch_contract_info(&self, client: &Client) -> Result<Option<ContractInfo>> {
        let info_contract_call =
            api::storage().contracts().contract_info_of(&self.contract);

        let contract_info_of = client
            .storage()
            .at(None)
            .await?
            .fetch(&info_contract_call)
            .await?;

        Ok(contract_info_of)
    }
    pub fn basic_display_format_contract_info(info: ContractInfo) {
        let convert_trie_id = hex::encode(info.trie_id.0);
        name_value_println!("TrieId:", format!("{}", convert_trie_id));
        name_value_println!("Code hash:", format!("{:?}", info.code_hash));
        name_value_println!("Storage items:", format!("{:?}", info.storage_items));
        name_value_println!(
            "Storage deposit:",
            format!("{:?}", info.storage_item_deposit)
        );
    }
    pub fn serialize_json(info: ContractInfo) {
        let convert_trie_id = hex::encode(info.trie_id.0);
        let info_to_json = InfoToJson {
            trie_id: convert_trie_id,
            code_hash: info.code_hash,
            storage_items: info.storage_items,
        };
        name_value_println!(
            "Test output json",
            format!("{:?}", serde_json::to_string_pretty(&info_to_json))
        );
    }
}

#[derive(serde::Serialize)]
struct InfoToJson {
    trie_id: String,
    code_hash: sp_core::H256,
    storage_items: u32,
}
pub enum OutputType {
    /// Output build results in a human readable format.
    HumanReadable,
    /// Output the build results JSON formatted.
    Json,
}
