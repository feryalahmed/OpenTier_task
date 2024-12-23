use crate::message::EchoMessage;
use log::{error, info, warn};
use prost::Message;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Client { stream }
    }

    pub fn handle(&mut self) -> io::Result<()> {
        let mut buffer = [0; 512];
        // Read data from the client
        let bytes_read = self.stream.read(&mut buffer)?;
        if bytes_read == 0 {
            info!("Client disconnected.");
            return Ok(());
        }

        if let Ok(message) = EchoMessage::decode(&buffer[..bytes_read]) {
            info!("Received: {}", message.content);
            // Echo back the message
            let payload = message.encode_to_vec();
            self.stream.write_all(&payload)?;
            self.stream.flush()?;
        } else {
            error!("Failed to decode message");
        }

        Ok(())
    }
}

pub struct Server {
    listener: TcpListener,
    is_running: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new server instance
    pub fn new(addr: &str) -> io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        let is_running = Arc::new(AtomicBool::new(false));
        Ok(Server {
            listener,
            is_running,
        })
    }

    /// Runs the server, listening for incoming connections and handling them
    pub fn run(&self) -> io::Result<()> {
        // Set the server as running
        self.is_running.store(true, Ordering::SeqCst); // Set the server as running
        info!("Server is running on {}", self.listener.local_addr()?);

        // Wrap the listener in an Arc<Mutex> for safe multi-threaded access
        let listener = Arc::new(Mutex::new(self.listener.try_clone()?));
        // Clone the `is_running` flag for use in the listener thread
        let is_running = Arc::clone(&self.is_running);

        // Spawn the listener thread to handle incoming connections
        thread::spawn(move || {
            while is_running.load(Ordering::SeqCst) {
                // Lock the listener to ensure exclusive access in this thread
                let listener = listener.lock().unwrap();

                // Attempt to accept a new client connection
                match listener.accept() {
                    Ok((stream, addr)) => {
                        info!("New client connected: {}", addr);
                        // Spawn a new thread to handle the connected client
                        let is_running = Arc::clone(&is_running);
                        thread::spawn(move || {
                            let mut client = Client::new(stream);

                            // Handle client communication while the server is running
                            while is_running.load(Ordering::SeqCst) {
                                if let Err(e) = client.handle() {
                                    if e.kind() == ErrorKind::ConnectionReset {
                                        info!("Client disconnected unexpectedly.");
                                    } else {
                                        error!("Error handling client: {}", e);
                                    }
                                    break;
                                }
                            }
                        });
                    }
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        // No incoming connections, sleep briefly to reduce CPU usage
                        thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        error!("Error accepting connection: {}", e);
                    }
                }
            }
        });
        // Log that the server has stopped when the function exits
        info!("Server stopped.");
        Ok(())
    }
    
    /// Stops the server by setting the is_running flag to false
    pub fn stop(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
            info!("Shutdown signal sent.");
        } else {
            warn!("Server was already stopped or not running.");
        }
    }
}