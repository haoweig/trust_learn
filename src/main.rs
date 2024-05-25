use std::io;

fn main() -> io::Result<()>{
    let nic = tun_tap::Iface::new("tun0", tun_tap::Mode::Tun).expect("Failed to cr");
    let mut buf = [0u8; 1504];
    loop {
        let nbytes = nic.recv(&mut buf [..])?;
        let eth_flags = u16::from_be_bytes([buf[0],buf[1]]);
        let eth_proto = u16::from_be_bytes([buf[2],buf[3]]);
        if eth_proto != 0x0800 {
            // no ipv4
            continue;
        }

        match etherparse::Ipv4HeaderSlice::from_slice(&buf[4..nbytes])  {
            Ok(p)=> {
                let src_ip = p.source_addr();
                let dst_ip = p.destination_addr();
                let proto = p.protocol();
                if proto != 0x06 {
                    // not tcp
                    continue;
                }

                match etherparse::TcpHeaderSlice::from_slice(&buf[4+p.slice().len()..]) {
                    Ok(p)=>{
                        eprintln!("{} → {} {}bytes of tcp to port {}", src_ip, dst_ip, p.slice().len(),p.destination_port());
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
