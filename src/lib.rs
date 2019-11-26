use failure::Error;
use std::collections::HashMap;
use ssh2::{Session, ScpFileStat, Channel};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::net::TcpStream;
use std::path::Path;
use std::str;
use std::string::String;

const SCPMODE: i32 = 0o644; // chmod 644

pub struct SSH {
    session: Option<Session>,
    host: String,
    port: u16,
}

impl SSH {

    /// Creates an SSH object with the host/IP and port. The connection is not established at this point.
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            session: None,
            host: host.to_owned(),
            port: port,
        }
    }

    /// Returns a reference to self.session. This is to clean up code in other functions.
    fn sess_ref(&self) -> &Session {
        self.session.as_ref().unwrap()
    }

    /// Create a TCP socket and establish handshake with server.
    fn create_socket(&self) -> Result<Session, Error> {
        let socket = TcpStream::connect(format!("{}:{}", self.host, self.port))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(socket);

        // Handshake and authentication
        sess.handshake()?;
        Ok(sess)
    }

    /// Returns list of identities known in `ssh-agent`
    pub fn identities() -> Result<HashMap<String, Vec<u8>>, Error> {
        let sess = Session::new().unwrap();
        let mut agent = sess.agent().unwrap();

        // Connect the agent and request a list of identities
        agent.connect().unwrap();
        agent.list_identities().unwrap();

        let mut identities = HashMap::new();
        for identity in agent.identities() {
            let identity = identity.unwrap(); // assume no I/O errors
            identities.insert(
                identity.comment().to_owned(),
                identity.blob().to_owned());
        }
        Ok(identities)
    }

    /// Initialize connection and authenticate to SSH server
    pub fn connect(&mut self, username: &str, pass: &str) -> Result<(), Error> {
        let sess = self.create_socket()?;
        sess.userauth_password(username, pass)?;
        assert!(sess.authenticated()); // Convert this to `Error` type
        self.session = Some(sess);
        Ok(())
    }

    /// Authenticate using `ssh-agent`.
    /// This allows for use of public key instead of username and password.
    pub fn connect_agent(&mut self, username:&str) -> Result<(), Error> {
        let sess = self.create_socket()?;
        sess.userauth_agent(username)?;
        assert!(sess.authenticated()); // Convert this to `Error` type
        self.session = Some(sess);
        Ok(())
    }

    /// Returns a bool based on status of authentication.
    pub fn authed(&self) -> bool {
        self.sess_ref().authenticated()
    }

    /// Keepalive settings.
    /// Reply determines if we want a response from server
    /// Interval is the number of seconds
    pub fn keepalive(&mut self, reply: bool, interval: u32) -> Result<(), Error> {
        self.sess_ref().set_keepalive(reply, interval);
        self.sess_ref().keepalive_send()?;
        Ok(())
    }

    /// An SSH tunnel. Unfortunately this is not functional as of right now.
    pub fn tunnel(&mut self, host: &str, port: u16, dst: Option<(&str, u16)>) -> Result<Channel, Error> {
        assert_eq!(self.authed(), true);
        let sess = self.sess_ref();
        let channel = sess.channel_direct_tcpip(host, port, dst).unwrap();
        Ok(channel)
    }


    /// SSH forwarding. Not functional.
    pub fn forward(&mut self, host: &str, port: u16, dst: Option<(&str, u16)>) -> Result<Channel, Error> {
        assert_eq!(self.authed(), true);
        let sess = self.sess_ref();
        let channel = sess.channel_direct_tcpip(host, port, dst).unwrap();
        Ok(channel)
    }

    /// Still a work in progress for interactive shell.
    pub fn get_shell(&self) -> Result<(), Error> {
        let mut channel = self.sess_ref().channel_session()?;
        channel.request_pty("xterm", None, None)?;
        channel.shell()?;
        channel.close()?;
        Ok(())
    }

    /// Run a command on the server.
    pub fn run_command(&self, cmd: &str) -> Result<String, Error> { 
        // Would be interesting to use fn(Channel)->String here
        let mut channel = self.sess_ref().channel_session()?;
        channel.exec(cmd)?;
        let mut stdout = String::new();
        channel.read_to_string(&mut stdout)?;
        Ok(stdout)
    }

    /// SCP a file to the server.
    pub fn upload_file(&self, fpath: &Path, dest: &Path) -> Result<(), Error> {
        let sess = self.sess_ref();
        // Read file to SCP into u8 array and get it's length
        let file = File::open(fpath)?;
        let mut reader = BufReader::new(file);
        let data = reader.fill_buf()?;
        let data_len = data.len() as u64;

        // Transfer file
        sess.scp_send(dest, SCPMODE, data_len, None)
            .unwrap()
            .write(data)?;
        Ok(())
    }

    /// Retrieve a file from the server.
    pub fn get_file(&self, fpath: &Path) -> Result<(Vec<u8>, ScpFileStat), Error> {
        let sess = self.session.as_ref().unwrap();
        let (mut remote_file, stat) = sess.scp_recv(fpath)?;
        let mut contents = Vec::new();
        remote_file.read_to_end(&mut contents)?;
        Ok((contents, stat))
    }
}
