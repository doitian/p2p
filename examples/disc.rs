use env_logger;
use log::debug;

use fnv::FnvHashMap;
use std::collections::HashSet;

use futures::prelude::*;

use tentacle::{
    builder::{MetaBuilder, ServiceBuilder},
    context::ServiceContext,
    multiaddr::Multiaddr,
    service::{DialProtocol, ProtocolHandle, ProtocolMeta, ServiceError, ServiceEvent},
    traits::ServiceHandle,
    ProtocolId, SessionId,
};

use discovery::{AddressManager, Discovery, DiscoveryProtocol, MisbehaveResult, Misbehavior};

fn main() {
    env_logger::init();
    let meta = create_meta(1, 0);
    let mut service = ServiceBuilder::default()
        .insert_protocol(meta)
        .forever(true)
        .build(SHandle {});

    if std::env::args().nth(1) == Some("server".to_string()) {
        debug!("Starting server ......");
        let _ = service.listen("/ip4/127.0.0.1/tcp/1337".parse().unwrap());
        tokio::run(service.for_each(|_| Ok(())))
    } else {
        debug!("Starting client ......");

        let _ = service.dial(
            "/ip4/127.0.0.1/tcp/1337".parse().unwrap(),
            DialProtocol::All,
        );
        let _ = service.listen("/ip4/127.0.0.1/tcp/1338".parse().unwrap());
        tokio::run(service.for_each(|_| Ok(())))
    }
}

fn create_meta(id: ProtocolId, start: u16) -> ProtocolMeta {
    let addrs: HashSet<Multiaddr> = (start..start + 3333)
        .map(|port| format!("/ip4/127.0.0.1/tcp/{}", port).parse().unwrap())
        .collect();
    let mut peers = FnvHashMap::default();
    peers.insert(0, (100, addrs));
    let addr_mgr = SimpleAddressManager { peers };
    MetaBuilder::default()
        .id(id)
        .service_handle(move || {
            let discovery = Discovery::new(addr_mgr);
            ProtocolHandle::Callback(Box::new(DiscoveryProtocol::new(id, discovery)))
        })
        .build()
}

struct SHandle {}

impl ServiceHandle for SHandle {
    fn handle_error(&mut self, _env: &mut ServiceContext, error: ServiceError) {
        debug!("service error: {:?}", error);
    }

    fn handle_event(&mut self, _env: &mut ServiceContext, event: ServiceEvent) {
        debug!("service event: {:?}", event);
    }
}

#[derive(Default, Clone, Debug)]
pub struct SimpleAddressManager {
    pub peers: FnvHashMap<SessionId, (i32, HashSet<Multiaddr>)>,
}

impl AddressManager for SimpleAddressManager {
    fn add_new_addr(&mut self, _session_id: SessionId, addr: Multiaddr) {
        let (_, addrs) = self
            .peers
            .entry(0)
            .or_insert_with(|| (100, HashSet::default()));
        addrs.insert(addr);
    }

    fn add_new_addrs(&mut self, session_id: SessionId, addrs: Vec<Multiaddr>) {
        for addr in addrs.into_iter() {
            self.add_new_addr(session_id, addr)
        }
    }

    fn misbehave(&mut self, _session_id: SessionId, _ty: Misbehavior) -> MisbehaveResult {
        let (score, _) = self.peers.entry(0).or_insert((100, HashSet::default()));
        *score -= 20;
        if *score < 0 {
            MisbehaveResult::Disconnect
        } else {
            MisbehaveResult::Continue
        }
    }

    fn get_random(&mut self, n: usize) -> Vec<Multiaddr> {
        self.peers
            .values()
            .flat_map(|(_, addrs)| addrs)
            .take(n)
            .cloned()
            .collect()
    }
}
