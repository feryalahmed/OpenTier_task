# Solution
### **Bug Details**:
- Issue: The server was not properly handling the ConnectionReset error when a client disconnected unexpectedly.
- Symptom: The test_client_echo_message test was failing.

### **Resolution**
- Fix: Add specific handling for the ConnectionReset error in the client.handle() loop.
- Updated Code Section:
  
``` rust
while self.is_running.load(Ordering::SeqCst) {
    if let Err(e) = client.handle() {
        if e.kind() == ErrorKind::ConnectionReset {
            info!("Client disconnected unexpectedly.");
        } else {
            error!("Error handling client: {}", e);
        }
        break;
    }
}
```
### **Outcome**
- The server now gracefully handles client disconnections.
- The test_client_echo_message test passes successfully.
- Improved server robustness by addressing unexpected client behavior.

### **Refactoring Server for Multi-Client Handling and Multi-Threading**
1- Shared Listener with Arc and Mutex:
The server was initially single-threaded, meaning it could only accept one client at a time. To handle multiple clients concurrently, the listener (responsible for accepting new connections) needed to be shared safely across multiple threads.
Solution: Used Arc (Atomic Reference Counted) and Mutex to wrap the listener. Arc allows multiple threads to own the listener safely, while Mutex ensures that only one thread can access the listener at a time.
``` rust
let listener = Arc::new(Mutex::new(self.listener.try_clone()?));
```
2- Listener Thread for Concurrent Connection Handling:
The server was previously blocked on a single connection. To accept multiple connections simultaneously, the listener needed to be moved to its own thread.
Solution: A new thread was spawned to listen for incoming connections, and this thread continuously checks for new clients while the server remains responsive.
``` rust
thread::spawn(move || {
    while is_running.load(Ordering::SeqCst) {
        let listener = listener.lock().unwrap();
        match listener.accept() {
            Ok((stream, addr)) => { ... }
        }
    }
});
```

3- Spawning Threads for Each Client:
Once a client connects, the server needed to handle the communication without blocking other clients. The server was updated to spawn a new thread for each client connection.
Solution: For each new client connection, a new thread is spawned to handle that client’s communication. This ensures the server can handle multiple clients simultaneously.
``` rust
thread::spawn(move || {
    let mut client = Client::new(stream);
    while is_running.load(Ordering::SeqCst) {
        if let Err(e) = client.handle() { ... }
    }
});
```

### **Results**
- All tests were successfully executed and passed individually, indicating that each feature works as expected in isolation. However, the tests do not run concurrently, which resulted in some inconsistencies when executing all tests together. Unfortunately, due to time constraints, I was unable to identify and resolve the underlying issue causing this behavior.

- Additionally, I was unable to address the final test due to time limitations. Despite these challenges, I made significant progress in understanding and working with the Rust programming language for the first time. This project has been an invaluable learning experience, and I have gained a deeper understanding of Rust’s capabilities and paradigms.

- Although I was not able to fully optimize the code and address all tests, I am confident that with more time, I could further refine the implementation and resolve the outstanding issues to ensure that all tests run smoothly together.

