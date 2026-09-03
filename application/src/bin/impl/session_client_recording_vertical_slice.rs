// Recording lifecycle browser vertical slice built on the existing application client.
use nc_pore_application::client::{ClientRole, ClientSessionError, ClientSessionService};
use nc_pore_application::distributed_recording::{begin_distributed_recording, confirm_distributed_recording_opening, mark_distributed_recording_ready, reconstitute_distributed_recording, DistributedRecording};
use nc_pore_application::distributed_recording_stop::acknowledge_distributed_recording_stop_in_core;
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{Recording, RecordingWorkflowStatus};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8788";
const SESSION_ID: &str = "vertical-slice-session";
const OWNER_ID: &str = "alice";
const RECORDING_ID: &str = "vertical-slice-recording";
const ARTIFACT_ID: &str = "vertical-slice-artifact";

struct Repo { sessions: Vec<ProductionSession> }
impl ProductionSessionRepository for Repo {
    type Error = &'static str;
    fn store(&mut self, s: &ProductionSession) -> Result<(), Self::Error> { self.sessions.push(s.clone()); Ok(()) }
    fn update(&mut self, s: &ProductionSession) -> Result<(), Self::Error> { *self.sessions.iter_mut().find(|x| x.id == s.id).ok_or("session not found")? = s.clone(); Ok(()) }
    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> { Ok(self.sessions.iter().find(|s| &s.id == id).cloned()) }
}
struct State { repo: Repo, recording: Option<DistributedRecording> }

fn main() -> std::io::Result<()> {
    let mut repo = Repo { sessions: Vec::new() };
    let mut client = ClientSessionService::new(&mut repo);
    client.create(SESSION_ID, OWNER_ID).expect("session creation");
    drop(client);
    let listener = TcpListener::bind(ADDRESS)?;
    println!("NC-PoRe recording slice: http://{ADDRESS}/?actor=alice");
    println!("NC-PoRe recording slice: http://{ADDRESS}/?actor=bob");
    let mut state = State { repo, recording: None };
    for stream in listener.incoming() { if let Ok(stream) = stream { handle(stream, &mut state); } }
    Ok(())
}

fn handle(mut stream: TcpStream, state: &mut State) {
    let mut buf = [0_u8; 16384];
    let n = match stream.read(&mut buf) { Ok(n) => n, Err(_) => return };
    let request = String::from_utf8_lossy(&buf[..n]);
    let mut sections = request.splitn(2, "\r\n\r\n");
    let head = sections.next().unwrap_or_default();
    let body = sections.next().unwrap_or_default();
    let mut line = head.lines().next().unwrap_or_default().split_whitespace();
    let method = line.next().unwrap_or_default();
    let path = line.next().unwrap_or_default();
    let result = if method == "GET" && path == "/" { (200, "text/html; charset=utf-8", HTML.to_owned()) }
    else if method == "GET" && path.starts_with("/api/state") { state_json(state) }
    else if method == "POST" && path.ends_with("/join") { join(state, body) }
    else if method == "POST" && path.ends_with("/start") { start(state) }
    else if method == "POST" && path.ends_with("/recording/begin") { begin(state, body) }
    else if method == "POST" && path.ends_with("/recording/ready") { ready(state, body) }
    else if method == "POST" && path.ends_with("/recording/open") { open_recording(state, body) }
    else if method == "POST" && path.ends_with("/recording/opening-confirm") { opening_confirm(state, body) }
    else if method == "POST" && path.ends_with("/recording/stop") { stop(state, body) }
    else if method == "POST" && path.ends_with("/recording/stop-ack") { stop_ack(state, body) }
    else if method == "POST" && path.ends_with("/recording/complete") { complete(state, body) }
    else { json(404, "not_found") };
    let response = format!("HTTP/1.1 {status} {reason}\r\nContent-Type: {ty}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", result.2.len(), status=result.0, reason=reason(result.0), ty=result.1, body=result.2);
    let _ = stream.write_all(response.as_bytes());
}

fn join(state: &mut State, body: &str) -> (u16, &'static str, String) {
    let id = field(body, "participant_id"); if id.is_none() { return json(400, "invalid_request"); }
    let mut client = ClientSessionService::new(&mut state.repo);
    match client.add_participant(SESSION_ID, OWNER_ID, &id.unwrap(), [ClientRole::Participant]) { Ok(_) => json(200, "ok"), Err(ClientSessionError::ParticipantAlreadyExists) => json(409, "participant_already_exists"), Err(_) => json(409, "join_rejected") }
}
fn start(state: &mut State) -> (u16, &'static str, String) { let mut client=ClientSessionService::new(&mut state.repo); if client.start(SESSION_ID,OWNER_ID).is_ok(){json(200,"ok")}else{json(409,"start_rejected")} }
fn begin(state:&mut State,body:&str)->(u16,&'static str,String){let actor=field(body,"actor_id");if actor.is_none(){return json(400,"invalid_request")}let actor=ParticipantId::new(actor.unwrap());if actor.value()!=OWNER_ID{return json(403,"owner_required")}if state.recording.is_some(){return json(409,"recording_already_active")}let pid=ProductionId::new(SESSION_ID);let rid=nc_pore_core::recording::RecordingId::new(RECORDING_ID);let mut session=state.repo.get(&pid).ok().flatten();if session.is_none(){return json(404,"session_not_found")}let mut session=session.take().unwrap();if !session.recordings().iter().any(|r|r.id()==&rid){if session.add_recording_by(&actor,Recording::new(RECORDING_ID)).is_err(){return json(409,"recording_rejected")}if state.repo.update(&session).is_err(){return json(500,"repository_error")}}match begin_distributed_recording(&mut state.repo,&pid,&actor,&rid){Ok(r)=>{let out=recording_json(&r);state.recording=Some(r);(200,"application/json; charset=utf-8",out)},Err(_)=>json(409,"recording_begin_rejected")}}
fn ready(state:&mut State,body:&str)->(u16,&'static str,String){let id=field(body,"participant_id");if id.is_none(){return json(400,"invalid_request")}let p=ParticipantId::new(id.unwrap());let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")};if mark_distributed_recording_ready(&mut state.repo,r,&p).is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"ready_rejected")}}
fn open_recording(state:&mut State,body:&str)->(u16,&'static str,String){if field(body,"actor_id").as_deref()!=Some(OWNER_ID){return json(403,"owner_required")}let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")};if r.trigger_opening().is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"opening_rejected")}}
fn opening_confirm(state:&mut State,body:&str)->(u16,&'static str,String){let id=field(body,"participant_id");if id.is_none(){return json(400,"invalid_request")}let p=ParticipantId::new(id.unwrap());let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")};if confirm_distributed_recording_opening(&mut state.repo,r,&p).is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"opening_confirmation_rejected")}}
fn stop(state:&mut State,body:&str)->(u16,&'static str,String){let id=field(body,"actor_id");if id.is_none(){return json(400,"invalid_request")}let a=ParticipantId::new(id.unwrap());if a.value()!=OWNER_ID{return json(403,"owner_required")}let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")};let mut s=state.repo.get(&ProductionId::new(SESSION_ID)).ok().flatten();if s.is_none(){return json(404,"session_not_found")}let mut s=s.take().unwrap();if s.stop_recording_by(&a,r.recording_id()).is_err(){return json(409,"stop_rejected")}if state.repo.update(&s).is_err(){return json(500,"repository_error")}let rid=r.recording_id().clone();match reconstitute_distributed_recording(&state.repo,&ProductionId::new(SESSION_ID),&a,&rid){Ok(n)=>{*r=n;(200,"application/json; charset=utf-8",recording_json(r))},Err(_)=>json(409,"reconstitution_failed")}}
fn stop_ack(state:&mut State,body:&str)->(u16,&'static str,String){let id=field(body,"participant_id");if id.is_none(){return json(400,"invalid_request")}let p=ParticipantId::new(id.unwrap());let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")};if acknowledge_distributed_recording_stop_in_core(&mut state.repo,r,&p).is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"stop_ack_rejected")}}
fn complete(state:&mut State,body:&str)->(u16,&'static str,String){let id=field(body,"actor_id");if id.is_none(){return json(400,"invalid_request")}let a=ParticipantId::new(id.unwrap());if a.value()!=OWNER_ID{return json(403,"owner_required")}let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")};let c=r.workflow().coordination();if r.workflow().status()!=RecordingWorkflowStatus::Stopping||c.stop_acknowledged_participants().len()!=c.participants().len(){return json(409,"stop_ack_barrier_not_complete")}let mut s=state.repo.get(&ProductionId::new(SESSION_ID)).ok().flatten();if s.is_none(){return json(404,"session_not_found")}let mut s=s.take().unwrap();if s.complete_recording_by(&a,r.recording_id(),nc_pore_core::recording::RecordingArtifactId::new(ARTIFACT_ID)).is_err(){return json(409,"complete_rejected")}if state.repo.update(&s).is_err(){return json(500,"repository_error")}let rid=r.recording_id().clone();match reconstitute_distributed_recording(&state.repo,&ProductionId::new(SESSION_ID),&a,&rid){Ok(n)=>{*r=n;(200,"application/json; charset=utf-8",recording_json(r))},Err(_)=>json(409,"reconstitution_failed")}}

fn state_json(state:&mut State)->(u16,&'static str,String){if let Some(r)=state.recording.as_ref(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(200,"no_recording")}}
fn recording_json(r:&DistributedRecording)->String{let c=r.workflow().coordination();let list=|ids:&[ParticipantId]|ids.iter().map(|p|format!("\"{}\"",escape(p.value()))).collect::<Vec<_>>().join(",");format!("{{\"recording_status\":\"{:?}\",\"workflow\":\"{:?}\",\"participants\":[{}],\"ready\":[{}],\"opening_confirmed\":[{}],\"stop_acknowledged\":[{}]}}",r.workflow().recording().status(),r.workflow().status(),list(c.participants()),list(c.ready_participants()),list(c.opening_confirmed_participants()),list(c.stop_acknowledged_participants()))}
fn field(body:&str,name:&str)->Option<String>{let m=format!("\"{name}\":\"");let i=body.find(&m)?+m.len();let x=&body[i..];Some(x[..x.find('"')?].to_owned())}
fn escape(v:&str)->String{v.replace('\\',"\\\\").replace('"',"\\\"")}
fn json(status:u16,error:&'static str)->(u16,&'static str,String){(status,"application/json; charset=utf-8",format!("{{\"error\":\"{error}\"}}"))}
fn reason(s:u16)->&'static str{match s{200=>"OK",403=>"Forbidden",404=>"Not Found",409=>"Conflict",500=>"Internal Server Error",_=>"Bad Request"}}

const HTML:&str=r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>NC-PoRe recording slice</title><style>body{font:16px system-ui;max-width:900px;margin:2rem auto;padding:0 1rem}button{margin:.25rem;padding:.5rem .75rem}.box{border:1px solid #aaa;padding:1rem;margin:1rem 0}</style></head><body><h1>NC-PoRe recording client</h1><p>Open twice with <b>?actor=alice</b> and <b>?actor=bob</b>.</p><div class="box">Actor: <span id="actor"></span></div><div class="box"><pre id="state">—</pre></div><div id="buttons"></div><script>const q=new URLSearchParams(location.search),actor=q.get('actor')||'bob',b=document.getElementById('buttons'),state=document.getElementById('state');document.getElementById('actor').textContent=actor;async function post(path,data){const r=await fetch(path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data||{})});if(!r.ok)throw new Error((await r.json()).error);return r.json()}function button(label,path,data){const x=document.createElement('button');x.textContent=label;x.onclick=async()=>{try{await post(path,data);refresh()}catch(e){alert(e)}};b.appendChild(x)}async function refresh(){let r=await(await fetch('/api/state')).json();b.replaceChildren();state.textContent=JSON.stringify(r,null,2);if(r.error==='no_recording'){if(actor==='alice'){button('Start session','/start');button('Begin recording','/recording/begin',{actor_id:actor})}button('Join','/join',{participant_id:actor});return}if(r.workflow==='WaitingForReady'&&!r.ready.includes(actor))button('READY','/recording/ready',{participant_id:actor});if(actor==='alice'&&r.workflow==='Ready')button('Trigger Opening','/recording/open',{actor_id:actor});if(r.workflow==='Opening'&&!r.opening_confirmed.includes(actor))button('Confirm Opening','/recording/opening-confirm',{participant_id:actor});if(actor==='alice'&&r.workflow==='Recording')button('Stop recording','/recording/stop',{actor_id:actor});if(r.workflow==='Stopping'&&!r.stop_acknowledged.includes(actor))button('ACK stop','/recording/stop-ack',{participant_id:actor});if(actor==='alice'&&r.workflow==='Stopping'&&r.stop_acknowledged.length===r.participants.length)button('Complete recording','/recording/complete',{actor_id:actor})}setInterval(refresh,1000);refresh();</script></body></html>"#;
