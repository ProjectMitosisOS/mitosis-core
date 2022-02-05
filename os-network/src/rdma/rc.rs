use alloc::sync::Arc;

use KRdmaKit::device::RContext;
use KRdmaKit::qp::RC;

pub struct RCFactory<'a> { 
    rcontext: &'a RContext<'a>,
}

impl<'a> RCFactory<'a> { 
    pub fn new() -> Self { 
        unimplemented!();
    }
}

impl crate::ConnFactory for RCFactory<'_> { 
    type ConnMeta = super::ConnMeta;
    type ConnType = RCConn; 
    type ConnResult<T> =  super::ConnResult<T>; 

    fn create(&mut self, meta : Self::ConnMeta) 
        -> Self::ConnResult<Self::ConnType>
           where Self::ConnType : crate::Conn { 
        unimplemented!(); 
    }
}

pub struct RCConn {
    rc: Arc<RC>
}

impl crate::Conn for RCConn { 
    type IOResult<T> = super::IOResult<T>; 
    type ReqPayload =  u64;
    type CompPayload = u64; 

    fn post(&mut self, req : &Self::ReqPayload) -> Self::IOResult<()> { 
        unimplemented!();
    }

    fn poll(&mut self) -> Self::IOResult<Self::CompPayload> {
        unimplemented!();
    }


}