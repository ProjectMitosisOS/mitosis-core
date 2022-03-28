#[allow(dead_code)]
#[derive(Default,Debug, PartialEq, Eq)]
pub struct RegDescriptor {
    pub(crate) others: crate::bindings::StackRegisters,
    pub(crate) fs: u64,
    pub(crate) gs: u64,
}

impl RegDescriptor { 
    pub fn get_fs(&self) -> u64 { 
        self.fs
    }

    pub fn get_gs(&self) -> u64 { 
        self.gs
    }

    pub fn get_others_mut(&mut self) -> &mut crate::bindings::StackRegisters { 
        &mut self.others
    }

    pub fn get_fs_mut(&mut self) -> &mut u64 { 
        &mut self.fs
    }

    pub fn get_gs_mut(&mut self) -> &mut u64 { 
        &mut self.gs
    }    
}

impl os_network::serialize::Serialize for RegDescriptor {}
