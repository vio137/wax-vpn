//! Opt-in route observations. No persistent IP/results, credentials, redirects or telemetry.
use std::{collections::VecDeque, io::Read, net::IpAddr, sync::{Arc, atomic::{AtomicU64, Ordering}, mpsc::{channel, Receiver, Sender}}, time::{Duration, Instant}};
use egui::{RichText, Color32};
use serde::{Deserialize, Serialize};
use crate::theme;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub public_ip: bool,
    pub throughput: bool,
    pub latency: bool,
    pub speed_test: bool,
    pub ip_endpoint: String,
    pub latency_endpoint: String,
    pub speed_origin: String,
    pub interval_secs: u64,
}
impl Default for Config {
    fn default() -> Self { Self { public_ip:false, throughput:true, latency:false, speed_test:false,
        ip_endpoint:"https://speed.cloudflare.com/cdn-cgi/trace".into(),
        latency_endpoint:"https://speed.cloudflare.com/__down?bytes=0".into(),
        speed_origin:"https://speed.cloudflare.com".into(), interval_secs:30 } }
}
fn endpoint(raw: &str) -> Result<reqwest::Url, String> {
    let url = reqwest::Url::parse(raw).map_err(|_| "Enter a valid HTTPS URL")?;
    if url.scheme()!="https" || url.host_str().is_none() || !url.username().is_empty() || url.password().is_some() || url.fragment().is_some() {
        return Err("Use HTTPS without credentials or a fragment".into());
    }
    Ok(url)
}
fn client(seconds:u64) -> Result<reqwest::blocking::Client,String> {
    reqwest::blocking::Client::builder().timeout(Duration::from_secs(seconds))
        .connect_timeout(Duration::from_secs(3)).redirect(reqwest::redirect::Policy::none())
        .no_proxy().user_agent("waX-VPN/network-check").build().map_err(|_| "Could not start HTTPS check".into())
}
fn read_limit(response: reqwest::blocking::Response, max:usize) -> Result<Vec<u8>,String> {
    if !response.status().is_success() { return Err("Service returned an unsuccessful HTTP status".into()); }
    let mut data=Vec::new(); response.take((max+1) as u64).read_to_end(&mut data).map_err(|_| "Service response could not be read")?;
    if data.len()>max { return Err("Service response exceeded the size limit".into()); } Ok(data)
}
fn parse_ip(text:&str) -> Result<String,String> {
    let value=text.lines().find_map(|line|line.strip_prefix("ip=")).unwrap_or(text.trim());
    value.parse::<IpAddr>().map(|ip|ip.to_string()).map_err(|_| "Service did not return a valid IP address".into())
}
pub fn public_ip(url:&str) -> Result<String,String> {
    let data=read_limit(client(4)?.get(endpoint(url)?).header("Cache-Control","no-cache").send().map_err(|_| "IP lookup unavailable or timed out")?,8192)?;
    parse_ip(std::str::from_utf8(&data).map_err(|_| "IP response was not text")?)
}
fn latency(url:&str) -> Result<f64,String> {
    let url=endpoint(url)?; let client=client(4)?; let start=Instant::now();
    let response=client.get(url).header("Cache-Control","no-cache").send().map_err(|_| "HTTP latency check unavailable or timed out")?;
    if !response.status().is_success() { return Err("HTTP latency service returned an error".into()); }
    Ok(start.elapsed().as_secs_f64()*1000.0)
}
fn mbps(bytes:u64, duration:Duration) -> f64 { bytes as f64*8.0/duration.as_secs_f64().max(0.001)/1_000_000.0 }
fn speed(origin:&str, epoch:&AtomicU64, id:u64) -> Result<(f64,f64),String> {
    let root=endpoint(origin)?;
    if root.path()!="/" || root.query().is_some() { return Err("Speed service must be a bare HTTPS origin supporting /__down and /__up".into()); }
    let client=client(15)?;
    const DOWN:usize=8_000_000; const UP:usize=2_000_000;
    let url=root.join("/__down?bytes=8000000").map_err(|_|"Invalid speed service")?;
    let start=Instant::now(); let mut response=client.get(url).header("Cache-Control","no-cache").send().map_err(|_|"Download test unavailable or timed out")?;
    if !response.status().is_success(){return Err("Download service returned an error".into());}
    let mut n=0usize; let mut buf=[0u8;16384];
    loop {
        if epoch.load(Ordering::Acquire)!=id {return Err("Test cancelled after settings or route changed".into());}
        let got=response.read(&mut buf).map_err(|_|"Download interrupted or timed out")?;
        if got==0{break;} n+=got; if n>DOWN {return Err("Download exceeded the 8 MB limit".into());}
    }
    if n!=DOWN{return Err("Download service returned an unexpected payload size".into());}
    let down=mbps(n as u64,start.elapsed());
    if epoch.load(Ordering::Acquire)!=id{return Err("Test cancelled after settings or route changed".into());}
    let url=root.join("/__up").map_err(|_|"Invalid speed service")?;
    let start=Instant::now(); let response=client.post(url).header("Content-Type","application/octet-stream").body(vec![0u8;UP]).send().map_err(|_|"Upload test unavailable or timed out")?;
    let _=read_limit(response,8192)?;
    Ok((down,mbps(UP as u64,start.elapsed())))
}
#[derive(Clone,Copy)] enum Kind{Ip,Latency,Speed}
enum Value{Ip(Result<String,String>),Latency(Result<f64,String>),Speed(Result<(f64,f64),String>)}
struct Event{epoch:u64,value:Value}
pub struct Monitor{
    pub before:Option<Result<String,String>>,
    after:Option<Result<String,String>>,
    ip_at:Option<Instant>,
    latency_value:Option<Result<f64,String>>,
    speed_value:Option<Result<(f64,f64),String>>,
    speed_at:Option<Instant>,
    tx:Sender<Event>,rx:Receiver<Event>,epoch:Arc<AtomicU64>,
    ip_busy:bool,latency_busy:bool,speed_busy:bool,
    context:Option<(u64,u8,Config)>,
    last_probe:Option<Instant>,last_stats:Option<(Instant,u64,u64)>,
    down:VecDeque<f64>,up:VecDeque<f64>,rtt:VecDeque<f64>,
}
impl Default for Monitor{
    fn default()->Self{let(tx,rx)=channel();Self{before:None,after:None,ip_at:None,latency_value:None,speed_value:None,speed_at:None,tx,rx,epoch:Arc::new(AtomicU64::new(0)),ip_busy:false,latency_busy:false,speed_busy:false,context:None,last_probe:None,last_stats:None,down:VecDeque::new(),up:VecDeque::new(),rtt:VecDeque::new()}}
}
fn push(q:&mut VecDeque<f64>,v:f64){if q.len()==60{q.pop_front();}q.push_back(v);}
impl Monitor{
    pub fn new_intent(&mut self){self.before=None;self.after=None;self.ip_at=None;self.down.clear();self.up.clear();self.rtt.clear();self.last_stats=None;self.epoch.fetch_add(1,Ordering::AcqRel);}
    pub fn tick(&mut self,cfg:&Config,generation:u64,phase:u8,visible:bool,rx:u64,tx:u64){
        let changed=self.context.as_ref().is_none_or(|(g,p,c)| *g!=generation || *p!=phase || c!=cfg);
        if changed{
            if self.context.as_ref().is_some_and(|(_,_,c)| c.ip_endpoint!=cfg.ip_endpoint) {self.before=None;}
            self.epoch.fetch_add(1,Ordering::AcqRel);self.context=Some((generation,phase,cfg.clone()));self.after=None;self.ip_at=None;self.last_probe=None;self.last_stats=None;self.latency_value=None;self.speed_value=None;self.speed_at=None;self.rtt.clear();self.down.clear();self.up.clear();
            if !cfg.public_ip{self.before=None;}
        }
        while let Ok(e)=self.rx.try_recv(){
            match &e.value{Value::Ip(_)=>self.ip_busy=false,Value::Latency(_)=>self.latency_busy=false,Value::Speed(_)=>self.speed_busy=false}
            if e.epoch!=self.epoch.load(Ordering::Acquire){continue;}
            match e.value{Value::Ip(v)=>{self.after=Some(v);self.ip_at=Some(Instant::now());},Value::Latency(v)=>{if let Ok(ms)=v{push(&mut self.rtt,ms);}self.latency_value=Some(v);},Value::Speed(v)=>{self.speed_value=Some(v);self.speed_at=Some(Instant::now());}}
        }
        if phase==2&&cfg.throughput{
            let now=Instant::now();
            if let Some((at,old_rx,old_tx))=self.last_stats{
                if now.duration_since(at)>=Duration::from_secs(1){
                    if rx>=old_rx&&tx>=old_tx{push(&mut self.down,mbps(rx-old_rx,now.duration_since(at)));push(&mut self.up,mbps(tx-old_tx,now.duration_since(at)));}
                    self.last_stats=Some((now,rx,tx));
                }
            }else{self.last_stats=Some((now,rx,tx));}
        }
        // One IP lookup after connect, never constant IP polling. Re-entry waits for an older worker.
        if phase==2&&cfg.public_ip&&self.after.is_none()&&!self.ip_busy{self.start(Kind::Ip,cfg);}
        if phase==2&&visible&&cfg.latency&&!self.latency_busy&&self.last_probe.is_none_or(|t|t.elapsed()>=Duration::from_secs(cfg.interval_secs.clamp(10,300))){self.start(Kind::Latency,cfg);}
    }
    fn start(&mut self,kind:Kind,cfg:&Config){
        let tx=self.tx.clone();let epoch=self.epoch.clone();let id=epoch.load(Ordering::Acquire);let cfg=cfg.clone();
        match kind{Kind::Ip=>self.ip_busy=true,Kind::Latency=>{self.latency_busy=true;self.last_probe=Some(Instant::now());},Kind::Speed=>self.speed_busy=true}
        std::thread::spawn(move||{
            let value=match kind{Kind::Ip=>Value::Ip(public_ip(&cfg.ip_endpoint)),Kind::Latency=>Value::Latency(latency(&cfg.latency_endpoint)),Kind::Speed=>Value::Speed(speed(&cfg.speed_origin,&epoch,id))};
            let _=tx.send(Event{epoch:id,value});
        });
    }
    pub fn ui(&mut self,ui:&mut egui::Ui,cfg:&Config,stable:bool,connected:bool){
        theme::section_heading(ui,"Network","Observations on the current route, not a leak test.");
        ui.add(egui::Label::new(RichText::new("IP, latency and speed checks contact the configured service (Cloudflare by default), which sees your public IP. Speed tests use up to 10 MB plus protocol overhead.").size(12.0).color(theme::SUB)).wrap());
        ui.add_space(10.0);
        if cfg.public_ip{
            theme::card_frame().show(ui,|ui|{
                ui.label(RichText::new("Public IP").font(theme::semi(15.0)));
                ui.label(format!("Before connection: {}",ip_label(&self.before)));
                ui.label(format!("{}: {}",if connected{"After connection"}else{"Current route"},ip_label(&self.after)));
                if let Some(at)=self.ip_at{ui.label(RichText::new(format!("Observed {}s ago. One address family; split routing can use another route.",at.elapsed().as_secs())).size(11.0).color(theme::MUTE));}
                if ui.add_enabled(stable&&!self.ip_busy,egui::Button::new(if self.ip_busy{"Checking IP..."}else{"Refresh public IP"})).clicked(){self.start(Kind::Ip,cfg);}
                if !connected {ui.label(RichText::new("Before-IP is captured on the next Connect; an already-running tunnel has no baseline.").size(11.0).color(theme::MUTE));}
            });ui.add_space(8.0);
        }
        if cfg.throughput{
            theme::card_frame().show(ui,|ui|{
                ui.label(RichText::new("Live tunnel throughput").font(theme::semi(15.0)));
                ui.label(format!("Received {:.2} Mbps   Sent {:.2} Mbps",self.down.back().unwrap_or(&0.0),self.up.back().unwrap_or(&0.0)));
                chart(ui,&self.down,Some(&self.up),"Received / sent, latest 60 one-second samples (adapter counters)");
                if !connected{ui.label(RichText::new("Connect to collect adapter-counter samples.").size(11.0).color(theme::MUTE));}
            });ui.add_space(8.0);
        }
        if cfg.latency{
            theme::card_frame().show(ui,|ui|{
                ui.label(RichText::new("HTTPS response latency").font(theme::semi(15.0)));
                let label=match &self.latency_value{Some(Ok(ms))=>format!("{ms:.0} ms"),Some(Err(e))=>e.clone(),None=>"No sample yet".into()};ui.label(label);
                chart(ui,&self.rtt,None,"Latest 60 successful samples (ms), including DNS/TCP/TLS; not ICMP ping");
                ui.label(RichText::new(format!("Automatic every {}s only while this page is visible and connected. Errors leave gaps in time, not zero latency.",cfg.interval_secs.clamp(10,300))).size(11.0).color(theme::MUTE));
                if ui.add_enabled(stable&&!self.latency_busy,egui::Button::new(if self.latency_busy{"Checking latency..."}else{"Test latency now"})).clicked(){self.start(Kind::Latency,cfg);}
            });ui.add_space(8.0);
        }
        if cfg.speed_test{
            theme::card_frame().show(ui,|ui|{
                ui.label(RichText::new("Quick speed test").font(theme::semi(15.0)));
                ui.label(RichText::new("One 8 MB download and 2 MB upload. Approximate single-request HTTPS speed, not line capacity. Uses the current OS route and may affect other traffic.").size(11.5).color(theme::SUB));
                if let Some(result)=&self.speed_value{match result{Ok((d,u))=>{ui.label(format!("Download {d:.2} Mbps   Upload {u:.2} Mbps"));},Err(e)=>{ui.label(RichText::new(e).color(theme::WARN));}}}
                if let Some(at)=self.speed_at{ui.label(RichText::new(format!("Measured {}s ago",at.elapsed().as_secs())).size(11.0).color(theme::MUTE));}
                if ui.add_enabled(stable&&!self.speed_busy,egui::Button::new(if self.speed_busy{"Testing speed..."}else{"Run speed test (10 MB)"})).clicked(){self.start(Kind::Speed,cfg);}
            });
        }
        if !cfg.public_ip&&!cfg.throughput&&!cfg.latency&&!cfg.speed_test{ui.label("All measurements are off. Enable them in Features > Network measurements.");}
        ui.add_space(8.0);ui.label(RichText::new("Change toggles and HTTPS service endpoints in Features > Network measurements. Results stay in memory and are not in local diagnostics.").size(11.5).color(theme::MUTE));
    }
}
fn ip_label(v:&Option<Result<String,String>>)->&str{match v{Some(Ok(ip))=>ip,Some(Err(e))=>e,None=>"Not measured"}}
fn chart(ui:&mut egui::Ui,a:&VecDeque<f64>,b:Option<&VecDeque<f64>>,caption:&str){
    let(r,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),72.0),egui::Sense::hover());
    ui.painter().rect_filled(r,egui::CornerRadius::same(6),theme::BG);
    let max=a.iter().chain(b.into_iter().flat_map(|q|q.iter())).copied().fold(0.1_f64,f64::max);
    for k in 1..3{let y=r.top()+r.height()*k as f32/3.0;ui.painter().line_segment([egui::pos2(r.left(),y),egui::pos2(r.right(),y)],egui::Stroke::new(0.5,theme::MUTE));}
    for(q,color)in std::iter::once((a,theme::GOOD)).chain(b.map(|q|(q,Color32::from_rgb(125,167,235)))){
        let points:Vec<_>=q.iter().enumerate().map(|(i,v)|egui::pos2(r.left()+r.width()*i as f32/59.0,r.bottom()-4.0-(r.height()-8.0)*(*v/max)as f32)).collect();
        if points.len()>1{ui.painter().add(egui::Shape::line(points,egui::Stroke::new(1.5,color)));}
    }
    ui.label(RichText::new(caption).size(10.5).color(theme::MUTE));
}
pub fn settings_ui(ui:&mut egui::Ui,cfg:&mut Config)->bool{
    let mut changed=false;
    ui.collapsing("Network measurements",|ui|{
        ui.label(RichText::new("Optional checks send your public IP to the configured HTTPS service. No signup, keys or results upload. Local throughput uses adapter counters only.").size(12.0).color(theme::SUB));
        changed|=theme::toggle_row(ui,&mut cfg.public_ip,"Public IP before / after","A bounded lookup before initial connect and once after connect; refresh manually.");
        changed|=theme::toggle_row(ui,&mut cfg.throughput,"Live throughput graph","Local adapter counters only; no extra network requests.");
        changed|=theme::toggle_row(ui,&mut cfg.latency,"Latency graph","HTTPS response time while Network is visible and connected.");
        changed|=theme::toggle_row(ui,&mut cfg.speed_test,"Manual speed test","Enable the test button. Never starts automatically; up to 10 MB per test.");
        ui.label("IP endpoint (trace ip=... or plain IP)");changed|=ui.text_edit_singleline(&mut cfg.ip_endpoint).changed();
        ui.label("Latency endpoint (small or empty HTTPS response)");changed|=ui.text_edit_singleline(&mut cfg.latency_endpoint).changed();
        ui.label("Speed service origin (Cloudflare-compatible /__down and /__up)");changed|=ui.text_edit_singleline(&mut cfg.speed_origin).changed();
        changed|=ui.add(egui::Slider::new(&mut cfg.interval_secs,10..=300).text("Latency interval (seconds)")).changed();
        for raw in [&cfg.ip_endpoint,&cfg.latency_endpoint,&cfg.speed_origin]{if let Err(e)=endpoint(raw){ui.label(RichText::new(e).color(theme::WARN));}}
    });changed
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn network_defaults_are_opt_in(){let c=Config::default();assert!(!c.public_ip&&!c.latency&&!c.speed_test);assert!(c.throughput);}
    #[test]fn endpoints_reject_insecure_credentials_and_fragments(){for x in ["http://example.com","https://a:b@example.com","https://example.com/#x","bad"]{assert!(endpoint(x).is_err());}assert!(endpoint("https://speed.cloudflare.com/__down?bytes=0").is_ok());}
    #[test]fn trace_and_plain_ip_parse_without_other_fields(){assert_eq!(parse_ip("fl=1\nip=1.2.3.4\nloc=XX").unwrap(),"1.2.3.4");assert_eq!(parse_ip("2001:db8::1\n").unwrap(),"2001:db8::1");assert!(parse_ip("<html>oops</html>").is_err());}
    #[test]fn history_bounded_and_units_correct(){let mut q=VecDeque::new();for i in 0..100{push(&mut q,i as f64);}assert_eq!(q.len(),60);assert_eq!(q[0],40.0);assert_eq!(mbps(1_000_000,Duration::from_secs(1)),8.0);}
    #[test]fn reset_clears_session_ip_and_history(){let mut m=Monitor::default();m.before=Some(Ok("1.2.3.4".into()));m.new_intent();assert!(m.before.is_none());assert!(m.after.is_none());}
    #[test]fn stale_results_and_counter_reset_do_not_publish(){
        let mut m=Monitor::default();let cfg=Config::default();
        m.tick(&cfg,1,2,false,100,100);
        let old=m.epoch.load(Ordering::Acquire);m.ip_busy=true;
        m.tx.send(Event{epoch:old,value:Value::Ip(Ok("1.2.3.4".into()))}).unwrap();
        m.tick(&cfg,2,0,false,0,0);assert!(m.after.is_none());assert!(!m.ip_busy);assert!(m.down.is_empty());
        assert!(!m.latency_busy&&!m.speed_busy);
    }
    #[test]#[ignore = "Manual opt-in: contacts Cloudflare, transfers up to 10 MB"]
    fn live_cloudflare_endpoints(){
        let cfg=Config::default();assert!(public_ip(&cfg.ip_endpoint).unwrap().parse::<IpAddr>().is_ok());
        assert!(latency(&cfg.latency_endpoint).unwrap()>0.0);
        let epoch=AtomicU64::new(0);let(d,u)=speed(&cfg.speed_origin,&epoch,0).unwrap();assert!(d.is_finite()&&d>0.0&&u.is_finite()&&u>0.0);
    }
    #[test]fn config_roundtrip_and_old_settings_default(){let c:Config=serde_json::from_str("{}").unwrap();assert_eq!(c,Config::default());assert_eq!(serde_json::from_str::<Config>(&serde_json::to_string(&c).unwrap()).unwrap(),c);}
}
