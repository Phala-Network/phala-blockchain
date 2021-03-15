use std::cell::RefCell;
use std::borrow::BorrowMut;
use crate::std::vec::Vec;
use crate::std::sync::SgxMutex;


lazy_static! {
    pub static ref  STATIC_MSG_TUNNEL_MUT:SgxMutex<MsgTunnel> = {
        let  m:MsgTunnel = MsgTunnel::default();
        SgxMutex::new(m)
    };
}


#[derive(Debug, Clone)]
pub struct MsgTunnel {
    list: RefCell<Vec<MsgTypeWrapper>>
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MsgType {
    BALANCE,
    WORKER,
}

#[derive(Debug, Clone)]
pub struct MsgTypeWrapper {
    pub msg_type: MsgType,
    pub msg_data: MsgData,
}


#[derive(Debug, Clone)]
pub struct MsgData {
    pub sequence: RefCell<u64>,
    pub queue: RefCell<Vec<RefCell<MsgDetail>>>,
}

#[derive(Debug, Clone)]
pub struct MsgDetail {
    pub msg_id: u64,
    pub body: Vec<u8>,
}

impl MsgTunnel {
    pub fn push(&mut self, msg_type: MsgType, message: Vec<u8>) {
        let list = &self.list;
        let mut insert_flag = false;
        for i in list.borrow_mut().iter() {
            let mut item_data = &i.msg_data;
            let item_type = i.msg_type;
            if msg_type == item_type {
                let mut sequence = item_data.borrow_mut().sequence.borrow_mut();
                let msg_id = sequence.borrow_mut().clone() + 1;
                *sequence += 1;
                let msg_detail = MsgDetail { msg_id, body: message.clone() };
                item_data.borrow_mut().queue.borrow_mut().push(RefCell::new(msg_detail));
                insert_flag = true;
            }
        }
        if !insert_flag{
            panic!("the msg_type hasn't been initialized");
        }
    }
    pub fn get_sequence(&mut self,msg_type:MsgType)->u64{
        let list = &self.list;
        let  seq = 0u64;
        for i in list.borrow_mut().iter() {
            let iter_msg_type = i.msg_type;
            let iter_sq =  i.msg_data.sequence.borrow_mut();
            if msg_type == iter_msg_type {
                return iter_sq.clone();
            }
        }
        return seq;
    }
   
    /// Called on received messages and drop them
    pub fn received(&mut self, msg_type: MsgType, seq: u64) {
        let list = &self.list;
        for i in list.borrow_mut().iter() {
            let item_data = &i.msg_data;
            let item_type = i.msg_type;
            if msg_type == item_type {
                let data_sequence = &item_data.sequence;
                if seq > *data_sequence.borrow_mut() {
                    return;
                }
                item_data.queue.borrow_mut().retain(|item| item.borrow_mut().msg_id > seq);
            }
        }
    }

    pub fn get_msg_type_wrapper(&mut self, msg_type: MsgType) -> Option<MsgTypeWrapper> {
        let list = &self.list;
        for i in list.borrow_mut().iter() {
            let item_type = i.msg_type;
            if msg_type == item_type {
                return Some(i.clone());
            }
        }
        None
    }

    pub fn get_msg_detail_list(&mut self,msg_type:MsgType,start_sequence:u64)->Option<Vec<MsgDetail>>{
        let wrappert_list = self.get_msg_type_wrapper(msg_type).unwrap();
        let msg_detail_list = wrappert_list.msg_data.queue;
        let mut v:Vec<MsgDetail> = Vec::new();
        for  i in msg_detail_list.borrow_mut().iter(){
            let item_msg_id = i.borrow_mut().msg_id;
            if item_msg_id>=start_sequence{
                v.push(i.borrow_mut().clone());
            }
        }
        Some(v)
    }
}

impl Default for MsgTunnel {
    fn default() -> Self {
        let msg_data = MsgData { sequence: RefCell::new(0u64), queue: RefCell::new(Vec::new()) };
        let balance_wrapper = MsgTypeWrapper { msg_type: MsgType::BALANCE, msg_data };
        let msg_data = MsgData { sequence: RefCell::new(0u64), queue: RefCell::new(Vec::new()) };
        let worker_wrapper = MsgTypeWrapper { msg_type: MsgType::WORKER, msg_data };
        let v = RefCell::new(Vec::new());
        v.borrow_mut().push(balance_wrapper);
        v.borrow_mut().push(worker_wrapper);
        Self {
            list: v,
        }
    }
}

