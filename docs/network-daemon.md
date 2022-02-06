### A short documentation of the network daemon

### Module graphs
```mermaid
flowchart LR
A[NetConn] --> B[RDMA]
B-->C[RC]
B-->D[UD]
B-->E[DCT]
F[RemoteMemory]
F --> S[RDMARM]
G[RPC]
```

