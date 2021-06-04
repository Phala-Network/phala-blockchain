use super::TransactionStatus;
use crate::std::collections::{HashMap, HashSet};
use crate::std::prelude::v1::*;
use crate::std::vec::Vec;
use csv_core::{ReadRecordResult, Reader};
use log::info;
use serde::{Deserialize, Serialize};

use crate::contracts;
use crate::types::TxRef;

pub type ItemId = u32;
pub type OrderId = u32;

// item

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Item {
    id: ItemId,
    txref: TxRef,
    seller: String,
    details: ItemDetails,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ItemDetails {
    pub name: String,
    pub category: String,
    pub description: String,
    pub price: PricePolicy,
    pub dataset_link: String,
    pub dataset_preview: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PricePolicy {
    PerRow {
        #[serde(with = "super::serde_balance")]
        price: chain::Balance,
    },
}

// order

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Order {
    id: OrderId,
    txref: TxRef,
    buyer: String,
    details: OrderDetails,
    state: OrderState, // maybe shouldn't serialize this
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderDetails {
    item_id: ItemId,
    query_link: String,
    // query parameters...
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct OrderState {
    data_ready: bool,
    query_ready: bool,
    result_ready: bool,
    matched_rows: u32,
    result_path: String,
}

// contract

#[derive(Serialize, Deserialize, Debug)]
pub enum Command {
    List(ItemDetails),
    OpenOrder(OrderDetails),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Request {
    GetItems,
    GetOrders,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    GetItems { items: Vec<Item> },
    GetOrders { orders: Vec<Order> },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPlaza {
    items: Vec<Item>,
    orders: Vec<Order>,
    #[serde(skip)]
    dataset: HashMap<String, Vec<u8>>,
}

impl DataPlaza {
    pub fn new() -> Self {
        Self {
            items: Vec::<Item>::new(),
            orders: Vec::<Order>::new(),
            dataset: HashMap::<String, Vec<u8>>::new(),
        }
    }

    pub fn set(&mut self, key: String, value: Vec<u8>) {
        self.dataset.insert(key, value);
    }

    pub fn get(&self, key: &String) -> Option<&Vec<u8>> {
        self.dataset.get(key)
    }

    fn update_order_state(&mut self) {
        for order in &mut self.orders {
            let item_id = order.details.item_id;
            let item = &self.items[item_id as usize];
            // check data available
            let data_link = &item.details.dataset_link;
            if self.dataset.contains_key(data_link) {
                order.state.data_ready = true;
            }
            // check query available
            let query_link = &order.details.query_link;
            if self.dataset.contains_key(query_link) {
                order.state.query_ready = true;
            }
            // compute if possible
            if order.state.data_ready && order.state.query_ready {
                let dataset = &self.dataset[data_link];
                let query = &self.dataset[query_link];
                let data = Self::compute(order, dataset, query);

                let path = order.state.result_path.clone();
                self.dataset.insert(path, data);
            }
        }
    }

    fn compute(order: &mut Order, dataset: &Vec<u8>, query: &Vec<u8>) -> Vec<u8> {
        // process query
        let mut targets = HashSet::<Vec<u8>>::new();
        let mut out = Vec::<u8>::new();
        let mut matched_rows = 0;
        {
            let mut first_line = true;
            let mut rdr = Reader::new();
            let mut bytes = query.as_slice();
            loop {
                let mut outbuf = [0; 2048];
                let mut ends = [0; 128];
                let (result, nin, _nout, nfield) = rdr.read_record(bytes, &mut outbuf, &mut ends);
                bytes = &bytes[nin..];
                match result {
                    ReadRecordResult::InputEmpty => {}
                    ReadRecordResult::OutputFull => panic!("record too large"),
                    ReadRecordResult::OutputEndsFull => panic!("too many fields"),
                    ReadRecordResult::Record => {
                        if first_line {
                            // do nothing, we assume the query has only one column
                            first_line = false;
                        } else {
                            // insert the query target to the hashset
                            if nfield == 1 {
                                // we only supports a single field right now
                                let value = Self::read_field(0, &outbuf, &ends).to_vec();
                                info!(
                                    "inserting query target: {}",
                                    String::from_utf8(value.clone()).unwrap()
                                );
                                targets.insert(value);
                            }
                        }
                    }
                    ReadRecordResult::End => break,
                }
            } // end loop
        }

        // process dataset
        {
            let mut first_line = true;
            let mut header_matched = false;
            let mut idx_phone = 0;

            let mut rdr = Reader::new();
            let mut bytes = dataset.as_slice();
            loop {
                let mut outbuf = [0; 2048];
                let mut ends = [0; 128];
                let (result, nin, _nout, nfield) = rdr.read_record(bytes, &mut outbuf, &mut ends);
                match result {
                    ReadRecordResult::InputEmpty => {}
                    ReadRecordResult::OutputFull => panic!("record too large"),
                    ReadRecordResult::OutputEndsFull => panic!("too many fields"),
                    ReadRecordResult::Record => {
                        if first_line {
                            // find the interested fields
                            for i in 0..nfield {
                                // let start = if i == 0 { 0 } else { ends[i - 1] };
                                // let field = &outbuf[start..ends[i]];
                                let field = Self::read_field(i, &outbuf, &ends);
                                if field == b"phone" {
                                    idx_phone = i;
                                    header_matched = true;
                                }
                            }
                            first_line = false;
                            if !header_matched {
                                panic!("query header doesn't match")
                            }
                        } else {
                            // try to match and output the entire line...
                            if idx_phone < nfield {
                                let value = Self::read_field(idx_phone, &outbuf, &ends).to_vec();
                                print!(
                                    "queryinig: {} - ",
                                    String::from_utf8(value.clone()).unwrap()
                                );
                                if targets.contains(&value) {
                                    info!("found");
                                    // should output the entire line!
                                    let mut full_line = (&bytes[..nin]).to_vec();
                                    out.append(&mut full_line);
                                    matched_rows += 1;
                                } else {
                                    info!("not found");
                                }
                            }
                        }
                    }
                    ReadRecordResult::End => break,
                }
                bytes = &bytes[nin..];
            } // loop
        }

        order.state.result_ready = true;
        order.state.matched_rows = matched_rows;
        order.state.result_path = format!("/order/{}", order.id);

        out
    }

    fn read_field<'a>(i: usize, outbuf: &'a [u8], ends: &[usize]) -> &'a [u8] {
        let start = if i == 0 { 0 } else { ends[i - 1] };
        let end = ends[i];
        &outbuf[start..end]
    }
}

impl contracts::Contract<Command, Request, Response> for DataPlaza {
    fn id(&self) -> contracts::ContractId {
        contracts::DATA_PLAZA
    }

    fn handle_command(
        &mut self,
        origin: &chain::AccountId,
        txref: &TxRef,
        cmd: Command,
    ) -> TransactionStatus {
        let address_hex = hex::encode(origin);
        let status = match cmd {
            Command::List(details) => {
                self.items.push(Item {
                    id: self.items.len() as ItemId,
                    txref: txref.clone(),
                    seller: address_hex,
                    details,
                });
                TransactionStatus::Ok
            }
            Command::OpenOrder(details) => {
                self.orders.push(Order {
                    id: self.orders.len() as OrderId,
                    txref: txref.clone(),
                    buyer: address_hex,
                    details,
                    state: OrderState {
                        // TODO
                        data_ready: false,
                        query_ready: false,
                        result_ready: false,
                        matched_rows: 0,
                        result_path: String::new(),
                    },
                });
                TransactionStatus::Ok
            }
        };

        status
    }

    fn handle_query(&mut self, _origin: Option<&chain::AccountId>, req: Request) -> Response {
        match req {
            Request::GetItems => Response::GetItems {
                items: self.items.clone(),
            },
            Request::GetOrders => {
                self.update_order_state();
                Response::GetOrders {
                    orders: self.orders.clone(),
                }
            }
        }
    }
}
