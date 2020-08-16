use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;
use tokio::stream::StreamExt;

pub struct Server {}

impl Server {
	pub fn new() -> Server {
		Server {}
	}

	pub async fn bind<A>(&mut self, addr: A) -> Result<(), tokio::io::Error>
		where A: ToSocketAddrs
	{
		let mut listener = TcpListener::bind(addr).await?;
		while let Some(stream) = listener.next().await {
			match stream {
				Ok(stream) => {
					println!("Connection with {} has established", stream.peer_addr()?);
				}
				Err(_) => { 
					println!("Connection with new client has failed");
				}
			}
			
		}
		Ok(())
	}
}