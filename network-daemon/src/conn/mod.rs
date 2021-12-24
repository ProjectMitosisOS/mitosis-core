pub mod rc;
pub mod dc;
pub mod ud;

/// RDMAConn is the abstract network connections of mitosis
pub trait RDMAConn { 
    // control path
    fn connect(); 

    // data path
    fn one_sided_read() ; // XD: should return a result
    fn one_sided_write(); 
    fn send_msg();
    fn recv_msg();     
}