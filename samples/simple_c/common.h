enum LibSwapCmd
{
    Nil = 0,  // for test only
    Dump = 4, // dump the memory mapping of this process
    Swap = 5, // swap to another process
    SwapRPC = 6, // swap to another process via RPC
};
