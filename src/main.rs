extern crate spring_dvs;
use spring_dvs::enums::*;
use spring_dvs::serialise::NetSerial;
use spring_dvs::protocol::{Packet, FrameRegister};

use std::env;
use std::net::UdpSocket;

fn modify_msg_type(arg: &str ) -> DvspMsgType {
	 
	match arg {
		"GsnRegistration" => DvspMsgType::GsnRegistration,
		"GsnResponse" => DvspMsgType::GsnResponse,
		_ => DvspMsgType::Undefined
	}
}

enum ArgState {
	None, MsgType, MsgContent, Target
}

fn main() {
	let mut msg_type = DvspMsgType::GsnRegistration;
	let mut msg_target: String =  "127.0.0.1:55301".to_string();
	let mut msg_content = String::new();
	let mut state: ArgState = ArgState::None;
	
	for a in env::args() {
		
		match a.as_ref() {
			
			"--type" => { state = ArgState::MsgType; },
			"--target" => { state = ArgState::Target; },
			"--content" => { state = ArgState::MsgContent; },
			
			_ => {
				
				match state {
					ArgState::MsgType => { msg_type = modify_msg_type(a.as_ref()); }
					ArgState::Target => { msg_target = a; }
					ArgState::MsgContent => { msg_content = a; }
					_ => { }
				};
				
				state = ArgState::None;
			}
		}
		
	}
	
	let bytes = forge_packet(msg_type, &msg_content);
	let socket = match UdpSocket::bind("0.0.0.0:55045") {
		Ok(s) => s,
		Err(e) => {
			println!("Error on bind: {}",e);
			return;
		}
	};
	
	let m : &str = msg_target.as_ref();
	
    match socket.send_to(bytes.as_ref(), m) {
    	Ok(_) => println!("Sent"),
    	_ => println!("Failed")
    };
}

fn forge_packet(t: DvspMsgType, c: &str) -> Vec<u8> {
	let bytes = match t {
		DvspMsgType::GsnRegistration => {
			let f = FrameRegister::new(true, DvspNodeType::Org as u8, DvspService::Dvsp, String::from(c));
			f.serialise()
		},
		_ => { Vec::new() }
	};
	
	let mut p = Packet::new(t);
	p.write_content(&bytes);
	
	p.serialise()
}