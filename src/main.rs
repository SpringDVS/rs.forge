extern crate spring_dvs;
use spring_dvs::enums::*;
use spring_dvs::serialise::NetSerial;
use spring_dvs::protocol::{Packet, PacketHeader, FrameRegister, FrameResponse};

use std::env;
use std::net::UdpSocket;

struct config {
	msg_type: DvspMsgType,
	msg_target: String,

	node_register: bool,
	node_type: DvspNodeType,
	node_state: DvspNodeState,
	node_service: DvspService,
	
	text_content: String,
	
}

impl config {
	fn new() -> config {
		config {
			msg_type: DvspMsgType::GsnRegistration,
			msg_target: "127.0.0.1:55301".to_string(),
			
			node_register: true,
			node_type: DvspNodeType::Org,
			node_state: DvspNodeState::Enabled,
			node_service: DvspService::Dvsp,
			
			text_content: String::new(),
		}
	}
}

fn modify_msg_type(arg: &str ) -> DvspMsgType {
	 
	match arg {
		"gsn_registration" => DvspMsgType::GsnRegistration,
		"gsn_response" => DvspMsgType::GsnResponse,
		_ => DvspMsgType::Undefined
	}
}

fn modify_node_register(arg: &str) -> bool {
	match arg {
		"true" => true,
		_ => false,
	}
}

fn modify_node_type(arg: &str) -> DvspNodeType {
	match arg {
		"org" => DvspNodeType::Org,
		"root" => DvspNodeType::Org,
		_ => DvspNodeType::Undefined,
	}
}

fn modify_node_state(arg: &str) -> DvspNodeState {
	match arg {
		"enabled" => DvspNodeState::Enabled,
		"disabled" => DvspNodeState::Disabled,
		"unresponsive" => DvspNodeState::Unresponsive,
		_ => DvspNodeState::Unspecified,
	}
}

fn modify_node_service(arg: &str) -> DvspService {
	match arg {
		"dvsp" => DvspService::Dvsp,
		"http" => DvspService::Http,
		_ => DvspService::Undefined,
	}
}

enum ArgState {
	None, MsgType, TextContent, MsgTarget, 
	NodeRegister, NodeType, NodeState, NodeService
}

fn main() {
	let mut cfg = config::new();
	
	let mut state: ArgState = ArgState::None;
	
	for a in env::args() {
		
		match a.as_ref() {
			
			"--msg-type" => { state = ArgState::MsgType; },
			"--msg-target" => { state = ArgState::MsgTarget; },
			
			
			"--node-type" => { state = ArgState::NodeType; },
			"--node-register" => { state = ArgState::NodeRegister; },
			"--node-service" => { state = ArgState::NodeService; },
			"--node-state" => { state = ArgState::NodeState; },
			
			"--text-content" => { state = ArgState::TextContent; },
			
			_ => {
				
				match state {
					ArgState::MsgType => { cfg.msg_type = modify_msg_type(a.as_ref()); },
					ArgState::MsgTarget => { cfg.msg_target = a; },
					ArgState::TextContent => { cfg.text_content = a; },
					
					ArgState::NodeRegister => { cfg.node_register = modify_node_register(a.as_ref()); },
					ArgState::NodeService => { cfg.node_service = modify_node_service(a.as_ref()); },
					ArgState::NodeType => { cfg.node_type = modify_node_type(a.as_ref()); },
					ArgState::NodeState => { cfg.node_state = modify_node_state(a.as_ref()); },
					_ => { }
				};
				
				state = ArgState::None;
			}
		}
		
	}
	
	let bytes = forge_packet(&cfg);
	println!("<< out.bytes.len: {}", bytes.len());
	let socket = match UdpSocket::bind("0.0.0.0:55045") {
		Ok(s) => s,
		Err(e) => {
			println!("Error on bind: {}",e);
			return;
		}
	};
	
	let m : &str = cfg.msg_target.as_ref();
	
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
   	
   	decode_packet(&bytes[..sz]);
   	println!("");
}

fn forge_packet(cfg: &config) -> Vec<u8> {
	let bytes = match cfg.msg_type {
		DvspMsgType::GsnRegistration => {
			let f = FrameRegister::new(cfg.node_register, cfg.node_type as u8, cfg.node_service, cfg.text_content.clone());
			f.serialise()
		},
		_ => { Vec::new() }
	};
	
	let mut p = Packet::new(cfg.msg_type);
	p.write_content(&bytes);
	
	println!("<< out.packet.msg_size: {}", p.header().msg_size);
	
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
	println!(">> in.bytes.len: {}", bytes.len());
	println!(">> in.packet.msg_size: {}\n", p.header().msg_size);
	println!(">> in.packet.msg_type: {:?}\n", p.header().msg_type);
	
	
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