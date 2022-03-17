use crate::bytes::BytesMut;
use KRdmaKit::mem::{Memory, RMemPhy};

/// UD must use physical address.
pub struct UDMsg {
    inner: RMemPhy,
    bytes: BytesMut,
    pa : u64
}

impl UDMsg {
    pub fn new_from_phy(mut phy: RMemPhy) -> Self {
        let pa = phy.get_pa(0); 
        Self {
            pa : pa,
            bytes: unsafe { BytesMut::from_raw(phy.get_ptr() as _, 
                phy.get_sz() as usize) },
            inner: phy,            
        }
    }

    pub fn new(size: usize) -> Self {
        Self::new_from_phy(RMemPhy::new(size))
    }

    pub fn get_bytes(&self) -> &BytesMut {
        &self.bytes
    }

    pub fn get_pa(&self) -> u64 {
        self.pa
    }

    pub fn len(&self) -> usize { 
        self.bytes.len()
    }    

    pub fn to_inner(self) -> RMemPhy {
        self.inner
    }
}

use core::fmt::{Arguments, Result, Write};

impl Write for UDMsg {
    #[inline]
    fn write_str(&mut self, s: &str) -> Result {
        self.bytes.write_str(s)
    }

    #[inline]
    fn write_fmt(&mut self, args: Arguments<'_>) -> Result {
        core::fmt::write(self, args)
    }
}
