use alloc::vec::Vec;

pub trait GetPhyAddr {
    fn get_physical_addr(self) -> crate::kern_wrappers::mm::PhyAddrType;
}

pub struct ShadowPageTable<P: GetPhyAddr> {
    table: Vec<P>,
}

impl<P> ShadowPageTable<P>
where
    P: GetPhyAddr,
{
    pub fn new() -> Self {
        Self { table: Vec::new() }
    }

    #[inline]
    pub fn add_page(&mut self, p : P) -> &mut Self { 
        self.table.push(p);
        self
    }

    pub fn len(&self) -> usize { 
        self.table.len()
    }
}
