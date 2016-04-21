extern crate spring_dvs;
extern crate rand;



use rand::Rng;

use spring_dvs::enums::*;
use spring_dvs::serialise::*;
use spring_dvs::protocol::{u8_packet_type, Ipv4};
use spring_dvs::protocol::{Packet, FrameRegister, FrameStateUpdate, FrameNodeRequest, FrameTypeRequest, FrameResolution, FrameRegisterGtn, FrameGeosub, FrameUnitTest};
use spring_dvs::protocol::{FrameResponse, FrameNodeInfo, FrameNodeStatus, FrameNetwork};
use spring_dvs::protocol::{HttpWrapper};
use spring_dvs::formats::ipv4_to_str_address;

use std::env;
use std::io::prelude::*;
use std::net::{TcpStream, UdpSocket};

struct Config {
	msg_type: DvspMsgType,
	msg_target: String,

	node_register: bool,
	node_type: DvspNodeType,
	node_state: DvspNodeState,
	node_service: DvspService,
	
	text_content: String,
	
	test_action: UnitTestAction,
	unit_test: bool,
	
	fuzzy: bool,
	fuzzy_loop: u32,
	fuzzy_valid_msg: bool,
	
	port: u32,
	
	http: bool,
	http_host: String,
	http_res: String,
	
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
			
			test_action: UnitTestAction::Undefined,
			unit_test: false,
			fuzzy: false,
			fuzzy_loop: 1,
			fuzzy_valid_msg: false,
			
			port: 57000,
			
			http: false,
			http_host: String::new(),
			http_res: String::new(),
			
		}
	}
}

fn modify_msg_type(arg: &str ) -> DvspMsgType {
	 
	match arg {
		"gsn_registration" => DvspMsgType::GsnRegistration,
		"gsn_response" => DvspMsgType::GsnResponse,
		"gsn_state_update" => DvspMsgType::GsnState,
		"gsn_node_status" => DvspMsgType::GsnNodeStatus,
		"gsn_resolution" => DvspMsgType::GsnResolution,
		"gsn_type_request" => DvspMsgType::GsnTypeRequest,
		
		"gsn_area" => DvspMsgType::GsnArea,

		"gsn_node_info" => DvspMsgType::GsnNodeInfo,
		
		"gtn_registration" => DvspMsgType::GtnRegistration,
		"gtn_geosub_nodes" => DvspMsgType::GtnGeosubNodes,
		
		"gsn_unit_test" => DvspMsgType::UnitTest,
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

fn modify_test_action(arg: &str) -> UnitTestAction {
	match arg {
		"reset" => UnitTestAction::Reset,
		"update-address" => UnitTestAction::UpdateAddress,
		"add-gsn-root" => UnitTestAction::AddGeosubRoot,
		_ => UnitTestAction::Undefined,
	}
}

enum ArgState {
	None, MsgType, TextContent, MsgTarget, 
	NodeRegister, NodeType, NodeState, NodeService,
	TestAction, FuzzyLoop, Port, Http,
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
			
			"--test-action" => { state = ArgState::TestAction; },
			"--unit-test" => { cfg.unit_test = true },
			"--fuzzy" => { cfg.fuzzy = true; cfg.unit_test = true },
			"--fuzzy-valid" => { cfg.fuzzy_valid_msg = true },
			"--fuzzy-loop" => { state = ArgState::FuzzyLoop },
			
			"--version" => { println!("SpringDVS Packet Forge v0.1"); return; }
			"--port" => { state = ArgState::Port }
			
			"--http" => { cfg.http = true; state = ArgState::Http; },

			_ => {
				
				match state {
					ArgState::MsgType => { cfg.msg_type = modify_msg_type(a.as_ref()); },
					ArgState::MsgTarget => { cfg.msg_target = a; },
					ArgState::TextContent => { cfg.text_content = a; },
					
					ArgState::NodeRegister => { cfg.node_register = modify_node_register(a.as_ref()); },
					ArgState::NodeService => { cfg.node_service = modify_node_service(a.as_ref()); },
					ArgState::NodeType => { cfg.node_type = modify_node_type(a.as_ref()); },
					ArgState::NodeState => { cfg.node_state = modify_node_state(a.as_ref()); },
					
					ArgState::TestAction => { cfg.test_action = modify_test_action(a.as_ref()); },
					ArgState::FuzzyLoop => { cfg.fuzzy_loop = match a.parse::<u32>() {
													Ok(n) => n,
													_ => 1
												}
										
							 				},
					ArgState::Port => { cfg.port = match a.parse::<u32>() {
													Ok(n) => n,
													_ => 0
												}
										
							 				},
					ArgState::Http => {
						let (h, r) =	match a.find('/') {
							None => (a.as_ref(), ""),
							Some(p) => a.split_at(p)
						} ;
						
						cfg.http_host = String::from(h);
						cfg.http_res = String::from(&r[1..]);
					},
					_ => { }
				};
				
				state = ArgState::None;
			}
		}
		
	}
	

	let address : String =  match cfg.port { 
		0 => format!("0.0.0.0:0"),
		_ =>  format!("0.0.0.0:{}", cfg.port),
	};

	for _ in 0 .. cfg.fuzzy_loop {
		
		let bytes = match cfg.fuzzy {
			true  => forge_fuzzy_packet(&cfg),
			false => forge_packet(&cfg),
		};

		if !cfg.unit_test {
			println!("<< out.bytes.len: {}\n", bytes.len());
			println!("<< out.bytes:");
			print_packet(bytes.as_ref());
			println!("\n");
		}
		
		
		
		let addr_str : &str = address.as_ref();
		match cfg.http {
			false => {
				 if dvsp_request(&bytes, addr_str, &cfg).is_err() { return }
			},
			
			true => {
				http_request(&bytes, &cfg);
			}
		}
		

	   	println!("");
	   	if cfg.fuzzy == false { break }
	}
	
	if cfg.fuzzy == true {
		println!("\n----\nCompleted {} Fuzzing(s)", cfg.fuzzy_loop);
	}
}

#[allow(unused_variables)]
fn forge_fuzzy_packet(cfg: &Config) -> Vec<u8> {
	let mut rng = rand::thread_rng();
	let mut sz = rng.gen::<usize>() % 2048;
	println!("Fuzzing: {} bytes", sz);
	
	if cfg.fuzzy_valid_msg && sz == 0 { 
		sz = (rng.gen::<usize>() % 2048) + 1;
	} 
	
	let mut v : Vec<u8> = Vec::new();
	for i in 0 .. sz {
		v.push(rng.gen::<u8>())
	}
	
	if cfg.fuzzy_valid_msg ==  true {
		
		while u8_packet_type(v[0]) == None {
			v[0] = rng.gen::<u8>()
		} 
	}
	
	println!("{:?}", v); 
	v
}

fn forge_packet(cfg: &Config) -> Vec<u8> {
	let bytes = match cfg.msg_type {
		DvspMsgType::GsnRegistration => {
			let f = FrameRegister::new(cfg.node_register, cfg.node_type as u8, cfg.node_service, cfg.text_content.clone());
			f.serialise()
		},
		
		DvspMsgType::GsnResolution => {
			let f = FrameResolution::new(&cfg.text_content);
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
		
		DvspMsgType::UnitTest => {
			let f = FrameUnitTest::new(cfg.test_action, &cfg.text_content);
			f.serialise()
		},
		
		DvspMsgType::GtnRegistration => {
			let f = FrameRegisterGtn::new(cfg.node_register, cfg.node_service, cfg.text_content.clone());
			f.serialise()
		},
		
		DvspMsgType::GtnGeosubNodes => {
			let f = FrameGeosub::new(&cfg.text_content);
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
		println!("%% Response|FrameResponse:{:?}", frame.code);
	}
}

fn decode_frame_node_info(frame: &FrameNodeInfo, cfg: &Config) {
	
	if frame.code != DvspRcode::Ok {

		if !cfg.unit_test {
			println!("FrameNodeInfo.code: {:?}", frame.code as u32);
		} else {
			println!("%% Response|FrameNodeInfo:{:?}", frame.code as u32);
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
		println!("%% Response|FrameNodeInfo:{};{:?};{};{}", frame.ntype, frame.service, ipv4_to_str_address(&frame.address), frame.name);
	}
	
}

fn decode_frame_node_status(frame: &FrameNodeStatus, cfg: &Config) {
	
	
	if frame.code != DvspRcode::Ok {
		if !cfg.unit_test  {
			println!("FrameNodeStatus.code: {:?}", frame.code);
		} else {
			println!("%% Response|FrameNodeStatus:{:?}", frame.code);
		}
		return;
	}
	
	if !cfg.unit_test {
		println!("FrameNodeStatus.status: {:?}", frame.status);
	} else {
		println!("%% Response|FrameNodeStatus:{:?}", frame.status);
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
		println!("%% Response|FrameNetwork:{}", nodelist);
	}

}

fn http_request(bytes: &Vec<u8>, cfg: &Config) -> Result<Success,Failure> {

	let serial = HttpWrapper::serialise_bytes_request(bytes, &cfg.http_host, &cfg.http_res);
	println!("REQUEST:\n{}", String::from_utf8(serial.clone()).unwrap());
	
	
	let mut stream = match TcpStream::connect(cfg.msg_target.as_str()) {
		Ok(s) => s,
		Err(_) => return Err(Failure::InvalidArgument)
	};
	
	stream.write(serial.as_slice());
	let mut buf = [0;4096];
	let size = match stream.read(&mut buf) {
				Ok(s) => s,
				Err(_) => 0
	};
	
	if size == 0 { return Err(Failure::InvalidArgument) }
	
	let bytes = match HttpWrapper::deserialise_response(Vec::from(&buf[0..size])) {
		Ok(p) => p,
		Err(_) => return Err(Failure::InvalidConversion)
	};
	
	decode_packet(bytes.as_slice(), &cfg);
	Ok(Success::Ok)

}

fn dvsp_request(bytes: &Vec<u8>, address: &str, cfg: &Config) -> Result<Success,Failure> {
	let socket = match UdpSocket::bind(address) {
		Ok(s) => s,
		Err(e) => {
			println!("!! Error on bind: {}",e);
			return Err(Failure::InvalidFormat);
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
			return Err(Failure::InvalidFormat); 
		},
	};
   	
   	decode_packet(&bytes[..sz], &cfg);
   	
   	Ok(Success::Ok)
}
