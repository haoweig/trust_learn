use std::io;

pub enum State{
    Closed,
    Listen,
    SynRcvd,
    // Estab,
}

struct SendSequenceSpace {
    una: u32,
    nxt: u32,
    wnd: u16,
    up: bool,
    wl1:usize,
    wl2:usize,
    iss:u32,
}

struct RecvSequenceSpace {
    nxt: u32,
    wnd: u16,
    up:bool,
    irs: u32,
}

pub struct Connection {
    state: State,
    send: SendSequenceSpace,
    recv: RecvSequenceSpace,
}

impl Connection {
    pub fn accept<'a> (
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>, 
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a[u8]) -> io::Result<Option<Self>>
    {
        let mut buf =[0u8;1500];
        if !tcph.syn(){
            //only expected syn packet
            return Ok(None);
        }
        let iss = 0;
        let mut c = Connection {
            state: State::SynRcvd,
            send: SendSequenceSpace {
                iss : iss,
                una : iss,
                nxt : iss + 1,
                wnd : 10,
                up: false,
                wl1: 0,
                wl2: 0,
            },
            recv: RecvSequenceSpace {
                irs : tcph.sequence_number(),
                nxt : tcph.sequence_number() + 1,
                wnd : tcph.window_size(),
                up: false,
            }
        };


        // need to start establishing a connection
        let mut syn_ack = etherparse::TcpHeader::new(tcph.destination_port(), 
        tcph.source_port(),
        c.send.iss, 
        c.send.wnd);
        syn_ack.acknowledgment_number = c.recv.nxt;
        syn_ack.syn = true;
        syn_ack.ack = true;
        let mut ip = etherparse::Ipv4Header::new(syn_ack.header_len(), 
        64, 
        etherparse::IpTrafficClass::Tcp, iph.destination_addr().octets(), iph.source_addr().octets());
        syn_ack.checksum = syn_ack.calc_checksum_ipv4(&ip,&[]).expect("failed to compute checksum");
        eprintln!("got ip header:\n{:02x?}", iph);
        eprintln!("got tcp header:\n{:02x?}", tcph);
        //write out the headers
        let unwritten = {
            let mut unwriten = &mut buf[..];
            ip.write(&mut unwriten);
            syn_ack.write(&mut unwriten);
            unwriten.len()
        };
        eprint!("responding with {:02x?}", &buf[..buf.len() - unwritten]);
        nic.send(&buf[..buf.len() - unwritten])?;
        Ok(Some((c)))
            
    }
    pub fn on_packet<'a> (
        &mut self,
        nic: &mut tun_tap::Iface,
        iph: etherparse::Ipv4HeaderSlice<'a>, 
        tcph: etherparse::TcpHeaderSlice<'a>,
        data: &'a[u8]) -> io::Result<()>{
            Ok(())
        }
}