extern crate spring_dvs;
use spring_dvs::enums::*;
use spring_dvs::serialise::NetSerial;
use spring_dvs::protocol::{Packet, PacketHeader, FrameRegister, FrameResponse};

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
    	Ok(_) => { },
    	_ => println!("Failed")
    };
    
    let mut bytes = [0;768];
   	let (sz, from) = match socket.recv_from(&mut bytes) {
		Ok(s) => s,
		_ => { 
			println!("Failed to recv response");
			return; 
		},
	};
   	
   	decode_packet(&bytes[..sz])
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


fn decode_packet(bytes: &[u8]) {
	let p : Packet = match Packet::deserialise(bytes) {
		Ok(p) => p,
		_ => { 
			println!("Failed to deserialise packet");
			return; 
		}
	};
	println!("byte.len: {}", bytes.len());
	println!("Packet.content.size: {}", p.header().msg_size);
	
	match p.header().msg_type {
		DvspMsgType::GsnResponse => {

			match p.content_as::<FrameResponse>() {
				Ok(frame) => decode_frame_response(&frame),
				Err(f) => {
					println!("Failed to deserialise frame: {:?}", f);
					return;
				} 
			}
		},
		
		_ => {
			println!("Unknown message type");
			return
		}
	}
}

fn decode_frame_response(frame: &FrameResponse) {
	println!("FrameResponse.code: {}", frame.code as u32);
}