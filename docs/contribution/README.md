# Contribution to Mitosis

Mitosis still has some unfinished designs and implementations, or you may also have your ideas. Welcome to contribute to mitosis!

## Unfinished Designs

### Fallback Path of Mitosis Page Handler

When there are pages swapped out of the memory, one-sided RDMA will fail to read the remote page. We need to use a RPC to get the remote page which lies on the disk.

### RPC Disconnection Handling

Clients should send disconnection requests to the server when they are about to exit. The server should handle the requests and remove local sessions.

### RPC Session Creation Error Handling

We need to handle the errors in RPC session creation (e.g.: Timeout).

### Doorbell optimization in Prefetcher

Prefetcher fetches several pages from remote machine in a round. Currently it reads the pages in a loop in a "read-and-poll" manner, which means that it will poll the completion after each read. This can be optimized with doorbell optimization.
