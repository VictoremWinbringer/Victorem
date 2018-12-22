use std::net::{UdpSocket, SocketAddr};


struct ClientSocket{
 socket:UdpSocket
}

struct ServerSocket{
 socket:UdpSocket
}

mod parser {
 use bincode::{serialize,deserialize};
 fn serialize_commands(commands: &crate::entities::CommandsPacket) -> Vec<u8> {
  serialize(commands).map_err(|e|crate::data_access_layer::logger::error(&format!("serialize_commands error {}",e)))
      .unwrap_or(Vec::new())
 }
}

pub mod logger{
    use log::{info,trace, warn, error};
    use simplelog::*;
    use std::fs::File;
 pub fn init(){
     CombinedLogger::init(
      vec![WriteLogger::new(LevelFilter::Info, Config::default(), File::create("victorem_framework_logs.log").unwrap())]
      );
  }
  pub fn error(error: &str){
        error!("{}",error)
    }

   pub fn warn(warn: &str){
        warn!("{}",warn)
    }
}
