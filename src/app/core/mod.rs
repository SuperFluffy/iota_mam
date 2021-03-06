use std::hash::{Hash, Hasher};

use crate::pb3::{Absorb, Result};
use crate::spongos::Spongos;
use crate::trits::{TritSlice, TritSliceMut, Trits};

pub const APPINST_SIZE: usize = 243;

#[derive(PartialEq, Eq, Clone)]
pub struct AppInst {
    pub(crate) id: Trits,
}

pub const MSGID_SIZE: usize = 81;

#[derive(PartialEq, Eq, Clone)]
pub struct MsgId {
    pub(crate) id: Trits,
}

impl Absorb for MsgId {
    fn wrap_absorb(&self, s: &mut Spongos, b: &mut TritSliceMut) {
        assert_eq!(MSGID_SIZE, self.id.size());
        self.id.wrap_absorb(s, b)
    }

    fn unwrap_absorb_sized(s: &mut Spongos, b: &mut TritSlice) -> Result<Self> {
        let mut id = Trits::zero(MSGID_SIZE);
        id.unwrap_absorb(s, b)?;
        Ok(MsgId { id })
    }
}

impl Hash for MsgId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub mod msg;
