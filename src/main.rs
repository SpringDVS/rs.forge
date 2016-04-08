extern crate spring_dvs;

use std::io::{Write};

use spring_dvs::enums::*;
use spring_dvs::serialise::NetSerial;
use spring_dvs::protocol::{Packet, FrameRegister, FrameStateUpdate, FrameNodeRequest, FrameTypeRequest};
use spring_dvs::protocol::{FrameResponse, FrameNodeInfo, FrameNodeStatus, FrameNetwork};
use spring_dvs::formats::ipv4_to_str_address;

use std::env;
use std::net::UdpSocket;

struct Config {
	msg_type: DvspMsgType,
	msg_target: String,

	node_register: bool,
	node_type: DvspNodeType,
	node_state: DvspNodeState,
	node_service: DvspService,
	
	text_content: String,
	
	unit_test: bool,
	
}

impl Config {
	fn new() -> Config {
		Config {
			msg_type: DvspMsgType::GsnRegistration,
			msg_target: "127.0.0.1:55301".to_string(),
			
			node_register: true,
			node_type: DvspNodeType::Org,
			node_state: DvspNodeState::Enabled,
			node_service: DvspService::Dvsp,
			
			text_content: String::new(),
			
			unit_test: false,
		}
	}
}

fn modify_msg_type(arg: &str ) -> DvspMsgType {
	 
	match arg {
		"gsn_registration" => DvspMsgType::GsnRegistration,
		"gsn_response" => DvspMsgType::GsnResponse,
		"gsn_state_update" => DvspMsgType::GsnState,
		"gsn_node_status" => DvspMsgType::GsnNodeStatus,

		"gsn_type_request" => DvspMsgType::GsnTypeRequest,
		
		"gsn_area" => DvspMsgType::GsnArea,

		"gsn_node_info" => DvspMsgType::GsnNodeInfo,
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
		"root" => DvspNodeType::Root,
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


fn print_packet(bytes: &[u8]) {
	for i in 0..14 {
		std::io::stdout().write(format!("{:0>2x} ", bytes[i]).as_ref()).unwrap();
	}
	if bytes.len() == 14 { return };
	
	std::io::stdout().write(format!("\n").as_ref()).unwrap();
	for i in 14 .. bytes.len() {
		std::io::stdout().write(format!("{:0>2x} ", bytes[i]).as_ref()).unwrap();
	}
}

fn main() {
	let mut cfg = Config::new();
	
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
			"--unit-test" => { cfg.unit_test = true },
			
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
	
	if !cfg.unit_test {
		println!("<< out.bytes.len: {}\n", bytes.len());
		println!("<< out.bytes:");
		print_packet(bytes.as_ref());
		println!("\n");
	}
	
	
	
	
	let socket = match UdpSocket::bind("0.0.0.0:55045") {
		Ok(s) => s,
		Err(e) => {
			println!("!! Error on bind: {}",e);
			return;
		}
	};
	
	let m : &str = cfg.msg_target.as_ref();
	
    match socket.send_to(bytes.as_ref(), m) {
    	Ok(_) => { },
    	_ => println!("!! Failed")
    };
    
    let mut bytes = [0;768];
   	let (sz, _) = match socket.recv_from(&mut bytes) {
		Ok(s) => s,
		_ => { 
			println!("!! Failed to recv response");
			return; 
		},
	};
   	
   	decode_packet(&bytes[..sz], &cfg);
   	println!("");
}

fn forge_packet(cfg: &Config) -> Vec<u8> {
	let bytes = match cfg.msg_type {
		DvspMsgType::GsnRegistration => {
			let f = FrameRegister::new(cfg.node_register, cfg.node_type as u8, cfg.node_service, cfg.text_content.clone());
			f.serialise()
		},
		
		DvspMsgType::GsnState => {
			let f = FrameStateUpdate::new(cfg.node_state, &cfg.text_content);
			f.serialise()
		},
		
		DvspMsgType::GsnNodeInfo => {
			let f = FrameNodeRequest::new(&cfg.text_content);
			f.serialise()
		},
		
		DvspMsgType::GsnNodeStatus => {
			let f = FrameNodeRequest::new(&cfg.text_content);
			f.serialise()
		},

		DvspMsgType::GsnTypeRequest => {
			let f = FrameTypeRequest::new(cfg.node_type as u8);
			f.serialise()
		},
		
		_ => { Vec::new() }
	};
	
	let mut p = Packet::new(cfg.msg_type);
	p.write_content(&bytes).unwrap();
	
	if !cfg.unit_test {
		println!("<< out.packet.msg_size: {}", p.header().msg_size);
	}
	
	p.serialise()
}


fn decode_packet(bytes: &[u8], cfg: &Config) {
	let p : Packet = match Packet::deserialise(bytes) {
		Ok(p) => p,
		_ => { 
			if !cfg.unit_test {
				println!("Failed to deserialise packet");
			} else {
				println!("## deserialise|Packet");
			}
			return; 
		}
	};
	
	if !cfg.unit_test {
		println!(">> in.bytes.len: {}", bytes.len());
		println!(">> in.packet.msg_size: {}", p.header().msg_size);
		println!(">> in.packet.msg_type: {:?}\n", p.header().msg_type);
		
		println!(">> in.bytes:");
		if bytes.len() < Packet::lower_bound() {
			println!("!! Error on byte len");
			return;
		}
	
		print_packet(bytes);
		println!("\n");
	}
	
	match p.header().msg_type {
		DvspMsgType::GsnResponse => {

			match p.content_as::<FrameResponse>() {
				Ok(frame) => decode_frame_response(&frame, &cfg),
				Err(f) => {
					println!("!! deserialise|FrameResponse:{:?}", f); 
					return;
				} 
			}
		},
		
		DvspMsgType::GsnResponseNodeInfo => {

			match p.content_as::<FrameNodeInfo>() {
				Ok(frame) => decode_frame_node_info(&frame, &cfg),
				Err(f) => {
					println!("!! deserialise|FrameNodeInfo:{:?}", f);
					return;
				} 
			}
		},
		
		DvspMsgType::GsnResponseStatus => {

			match p.content_as::<FrameNodeStatus>() {
				Ok(frame) => decode_frame_node_status(&frame, &cfg),
				Err(f) => {
					println!("!! deserialise|FrameNodeStatus:{:?}", f);
					return;
				} 
			}
		},
		
		DvspMsgType::GsnResponseNetwork => {

			match p.content_as::<FrameNetwork>() {
				Ok(frame) => decode_frame_network(&frame, &cfg),
				Err(f) => {
					println!("!! deserialise|FrameNetwork:{:?}", f);
					return;
				} 
			}
		},
		
		_ => {
			println!("!! unknown_message");
			return
		}
	}
}

fn decode_frame_response(frame: &FrameResponse, cfg: &Config) {
	if !cfg.unit_test {
		println!("FrameResponse.code: {:?}", frame.code);
	} else {
		println!("%% response|FrameResponse:{:?}", frame.code);
	}
}

fn decode_frame_node_info(frame: &FrameNodeInfo, cfg: &Config) {
	
	if frame.code != DvspRcode::Ok {

		if !cfg.unit_test {
			println!("FrameNodeInfo.code: {:?}", frame.code as u32);
		} else {
			println!("%% response|FrameNodeInfo:{:?}", frame.code as u32);
		}
		return;

	}
	
	if !cfg.unit_test {

		std::io::stdout().write(format!("FrameNodeInfo.type: ").as_ref()).unwrap();
		if frame.ntype == DvspNodeType::Undefined as u8 {
			std::io::stdout().write(format!("undefined").as_ref()).unwrap();
		} else {
			if frame.ntype & DvspNodeType::Org as u8 > 0 {
				std::io::stdout().write(format!("org ").as_ref()).unwrap();
			}
			
			if frame.ntype & DvspNodeType::Root as u8 > 0 {
				std::io::stdout().write(format!("root ").as_ref()).unwrap();
			}
		}
		
		println!("");
		
		println!("FrameNodeInfo.service: {:?}", frame.service);
		println!("FrameNodeInfo.address: {}", ipv4_to_str_address(&frame.address));
		println!("FrameNodeInfo.name: {}", frame.name);

	} else {
		println!("%% response|FrameNodeInfo:{};{:?};{};{}", frame.ntype, frame.service, ipv4_to_str_address(&frame.address), frame.name);
	}
	
}

fn decode_frame_node_status(frame: &FrameNodeStatus, cfg: &Config) {
	
	
	if frame.code != DvspRcode::Ok {
		if !cfg.unit_test  {
			println!("FrameNodeStatus.code: {:?}", frame.code);
		} else {
			println!("%% response|FrameNodeStatus:{:?}", frame.code);
		}
		return;
	}
	
	if !cfg.unit_test {
		println!("FrameNodeStatus.status: {:?}", frame.status);
	} else {
		println!("%% response|FrameNodeStatus:{:?}", frame.status);
	}
}

fn decode_frame_network(frame: &FrameNetwork, cfg: &Config) {
	
	
	let nodelist = String::from_utf8_lossy(frame.list.as_ref());
	
	if !cfg.unit_test {
		println!("FrameNetwork.list:");
		
		let atoms : Vec<&str> = nodelist.split(';').collect();
		for s in atoms {
			if s.len() == 0 { continue; }
			println!("{};", s);
		}
		
	} else {
		println!("%% response|FrameNetwork:{}", nodelist);
	}

}