use std::io;
use std::collections::HashMap;
use std::net::Ipv4Addr;

#[derive(Clone,Copy,Debug,Hash,Eq,PartialEq)]
struct Quad {
    src: (Ipv4Addr, u16),
    dst: (Ipv4Addr, u16),
}

mod tcp;

fn main() -> io::Result<()>{
    let mut connections: HashMap<Quad,tcp::Connection> = Default::default();
    let mut nic = tun_tap::Iface::without_packet_info("tun0", tun_tap::Mode::Tun).expect("Failed to cr");
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf [..])?;
        // let _eth_flags = u16::from_be_bytes([buf[0],buf[1]]);
        // let eth_proto = u16::from_be_bytes([buf[2],buf[3]]);
        // if eth_proto != 0x0800 {
        //     // no ipv4
        //     continue;
        // }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[..nbytes])  {
            Ok(iph)=> {
                let src_ip = iph.source_addr();
                let dst_ip = iph.destination_addr();
                if iph.protocol() != 0x06 {
                    // not tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[iph.slice().len()..]) {
                    Ok(tcph)=>{
                        use std::collections::hash_map::Entry; 
                        let datai = iph.slice().len() + tcph.slice().len();
                        match connections.
                            entry(Quad{
                                src: (src_ip, tcph.source_port()),
                                dst: (dst_ip, tcph.destination_port()),
                        }){
                            Entry::Occupied(mut c)=>{
                                c.get_mut().on_packet(&mut nic,iph,tcph, &buf[datai..nbytes])?;
                            },
                            Entry::Vacant(mut e)=>{
                                if let Some(c) = tcp::Connection::accept(&mut nic, iph, tcph, &buf[datai..nbytes])? {
                                    e.insert(c);
                                }
                            },
                        }
                    }
                    Err(e)=>
                    {
                        eprintln!("Ignoring weird tcp packet {:?}", e);
                    }
                }

            }
            Err(e)=>{
                eprintln!("Ignoring weird packet {:?}", e);
            }
        } 

        
    }
    Ok(())
}
