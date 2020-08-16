use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::net::TcpListener;
use tokio::net::ToSocketAddrs;
use tokio::stream::StreamExt;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{Value};

pub enum ServerError {
	Io(tokio::io::Error),
	Utf8(std::str::Utf8Error),
	ParseError(serde_json::error::Error),
}

impl From<tokio::io::Error> for ServerError {
	fn from(err: tokio::io::Error) -> Self {
		ServerError::Io(err)
	}
}

impl From<std::str::Utf8Error> for ServerError {
	fn from(err: std::str::Utf8Error) -> Self {
		ServerError::Utf8(err)
	}
}

impl From<serde_json::Error> for ServerError {
	fn from(err: serde_json::Error) -> Self {
		ServerError::ParseError(err)
	}
}

pub enum Request {
	Ping,
	Connect(ConnectRequest),
	Disconnect(DisconnectRequest),
	Get(GetRequest),
}

#[derive(Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct UserInfo {
	username: Option<String>,
	addr: Option<SocketAddr>,
}

#[derive(Serialize, Deserialize)]
pub enum VisibilityKind {
	All,
	FriendsOnly(Vec<UserInfo>),
	None,
}

#[derive(Serialize, Deserialize)]
pub struct ConnectRequest {
	info: UserInfo,
	visibility: VisibilityKind,
}

#[derive(Serialize, Deserialize)]
pub struct DisconnectRequest {
	info: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct GetRequest {
	local: UserInfo,
	target: UserInfo,
}

#[derive(Serialize, Deserialize)]
pub struct GetResponse {
	target: UserInfo,	
}

pub struct Server {
	clients: HashMap<UserInfo, VisibilityKind>,
}

impl Server {
	pub fn new() -> Server {
		Server {
			clients: HashMap::new()
		}
	}

	pub async fn bind<A>(&mut self, addr: A) -> Result<(), ServerError>
		where A: ToSocketAddrs
	{
		let mut listener = TcpListener::bind(addr).await?;
		while let Some(stream) = listener.next().await {
			match stream {
				Ok(mut stream) => {
					println!("Connection with {} has established", stream.peer_addr().map_err(|e| ServerError::Io(e))?);
					self.process(&mut stream).await?;
				}
				Err(_) => { 
					println!("Connection with new client has failed");
				}
			}
			
		}
		Ok(())
	}

	pub async fn process(&mut self, stream: &mut TcpStream) -> Result<(), ServerError> {
		let mut buffer = [0; 2048];
		stream.read(&mut buffer).await.map_err(|e| ServerError::Io(e))?;

		let request = std::str::from_utf8(&buffer).map_err(|e| ServerError::Utf8(e))?;
		let root: Value = serde_json::from_str(request)?;
		match root["type"].as_str() {
			Some(t) => {
				match t {
					"PING" => {
						if let Ok(result) = self.process_ping(Request::Ping).await {
							stream.write_all(result.as_bytes()).await?;
						}
					},
					"CONNECT" => {
						if let Ok(result) = self.process_connect(Request::Connect(serde_json::from_str(root["request"].as_str().unwrap())?)).await {
							stream.write_all(result.as_bytes()).await?;
						}
					},
					"DISCONNECT" => {
						if let Ok(result) = self.process_disconnect(Request::Disconnect(serde_json::from_str(root["request"].as_str().unwrap())?)).await {
							stream.write_all(result.as_bytes()).await?;
						}
					},
					"GET" => {

					},
					_ => {}
				}
			}
			_ => {}
		}

		Ok(())
	}

	pub async fn process_ping(&self, request: Request) -> Result<&str, ()> {
		if let Request::Ping  =	request {
			Ok("PONG")
		} else {
			Err(())
		}
	}

	pub async fn process_connect(&mut self, request: Request) -> Result<&str, ()> {
		if let Request::Connect(request) = request {
			self.clients.insert(
				request.info,
				request.visibility	
			);
			Ok("OK")
		} else {
			Err(())
		}
	}

	pub async fn process_disconnect(&mut self, request: Request) -> Result<&str, ()> {
		if let Request::Disconnect(request) = request {
			self.clients.remove(&request.info);
			Ok("OK")
		} else {
			Err(())
		}
	}

	pub async fn process_get(&mut self, request: Request) -> Result<GetResponse, &str> {
		if let Request::Get(request) = request {
			match self.clients.get(&request.target) {
				Some(visibility) => {
					match visibility {
						_ => {
							unimplemented!()
						}
					}
				},
				None => {
					Err("Invalid target")
				}
			} 
		} else {
			Err("Invalid request")
		}
	}
}