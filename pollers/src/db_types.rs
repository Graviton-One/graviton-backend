use serde::{
    Serialize,
    Deserialize
};
use diesel::prelude::*;
#[derive(Clone,DbEnum,Debug,Serialize,Deserialize,PartialEq,Copy)]
#[DieselType="ChainType"]
#[DbValueStyle="camelCase"]
pub enum ChainTypeEnum {
    Evm,  // All variants must be fieldless
    Sol,
}
