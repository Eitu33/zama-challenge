# Challenge Report

## Approach

I prioritized the most critical aspects of the project, carefully choosing how to allocate my time to ensure that the core functionality was operational before considering any additional optimizations. My main focus was on implementing exactly what was outlined in the project requirements. Only after completing those essentials did I address other improvements.

## Limitations

I set a strict 10-hour time limit for each test I perform. For this particular test, I used the full amount of time. Since I run multiple tests in parallel every week, it’s important to stick to this limit. It helps me avoid getting bogged down in implementation details, which I have a tendency to do if I don’t impose clear time constraints.

## Potential Improvements

- **Secure Transport**: Implement SSL or TLS to protect against data corruption during transmission.
  
- **Client-side Merkle Root Encryption**: Add encryption for local files to ensure confidentiality.
  
- **Public and Private Key Usage**: Introduce signing on the client side and store files on the server using public keys to prevent unauthorized file access by clients who didn’t upload them.

- **Server Stability**: Simulate attacks and ensure the server can handle crashes without stopping. Internal errors are acceptable but any error that halts the service is a major issue.

- **Error Handling**: Improve error handling on the server. Currently, many errors encountered after receiving a request are unwrapped and managed by the hyper service, but these errors could be more informative. Also some of them should be returned in the response payload.

- **Serialization with `nom`**: While `bincode` serialization is acceptable in this context, it would be essential to have more control over serialization. `nom` is usually my go-to crate for building generic and combinable parsers.

- **File Streaming**: Currently, files are read and transmitted in a single chunk. Using `hyper` ability to stream the body would improve efficiency. Additionally, files are saved with minimal organization and under a `.txt` extension, which could be significantly improved.

- **Merkle Tree Implementation**: I opted for a simple Merkle tree design that fills the leaves to the next power of two. While this approach mirrors Ethereum's implementation, it's not the most efficient. In the worst-case scenario, this requires hashing a number of leaves equivalent to `(((number of leaves rounded to the next power of two) / 2) - 1)`. A better approach could involve balancing the tree instead of simply filling it.
