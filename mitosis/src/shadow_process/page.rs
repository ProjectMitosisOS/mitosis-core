use crate::bindings::*;

/// A wrapper over the original linux's page data structure
/// It will copy the page to a newly allocated one to prevent overwritting 
/// Currently, we only support 4K pages
pub struct Copy4KPage { 
    inner : &'static mut page, // linux data structure wrapper always use the 'static lifetime 
}

impl Copy4KPage { 
    pub fn new() -> Self { 

    }
}