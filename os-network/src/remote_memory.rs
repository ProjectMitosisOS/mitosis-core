
pub trait Device { 
    // data for authentication the validity of the operation 
    type Key;

    // network address, e.g., IP    
    type Location;  

    // memory address 
    type Address;    

}