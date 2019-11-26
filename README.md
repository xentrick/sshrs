# sshrs

An SSH library written in Rust. This wraps the ssh2 bindings library and makes it easier to work with. This is simply meant to make quick and easy SSH connections while maintaining an active session.

Note: Forwarding, Tunnels, and interactive shell have not been fully implemented.

# Usage

Connection:

  ```
  let mut tunn = ssh::SSH::new(&HOST, 22);
  tunn.connect(&USR, &PWD).unwrap();
  assert_eq!(tunn.authed(), true);
  ```
  
Connection using ssh-agent:

  ```
  let mut tunn = ssh::SSH::new(&HOST, 22);
  tunn.connect_agent(&USR).unwrap();
  assert_eq!(tunn.authed(), true);
  ```
  
Execute command:

  ```
  let result = tunn.run_command("uname -a").unwrap();
  println!("Command Output: {}", result)
  ```
  
Upload a file:

  ```
  let src = Path::new("/home/ssh/important.txt");
  let dest = Path::new("/tmp/destination.txt");
  let result = tunn.upload_file(&src, dest);
  ```
  
Download a file:

  ```
  let fpath = Path::new("/tmp/downloadme.tar.gz");
  let (contents, stat) = tunn.get_file(fpath).unwrap();
  println!("File Contents: {}", String::from_utf8(contents).unwrap());
  ```
