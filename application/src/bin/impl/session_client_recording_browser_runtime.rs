use nc_pore_application::browser_recording_handoff::{
    BrowserRecordingHandoff, BrowserRecordingStopReason,
};
use nc_pore_application::client::{ClientRole, ClientSessionError, ClientSessionService};
use nc_pore_application::distributed_recording::{
    begin_distributed_recording, confirm_distributed_recording_opening,
    mark_distributed_recording_ready, reconstitute_distributed_recording, DistributedRecording,
};
use nc_pore_application::distributed_recording_stop::acknowledge_distributed_recording_stop_in_core;
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{Recording, RecordingWorkflowStatus};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use recorder::persistence::InMemoryPersistenceProvider;
use recorder::session::RecordingSessionId;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

const ADDRESS: &str = "127.0.0.1:8788";
const SESSION_ID: &str = "vertical-slice-session";
const OWNER_ID: &str = "alice";
const RECORDING_ID: &str = "vertical-slice-recording";
const ARTIFACT_ID: &str = "vertical-slice-artifact";
const MAX_REQUEST_BODY: usize = 8 * 1024 * 1024;

struct Repo {
    sessions: Vec<ProductionSession>,
}
impl ProductionSessionRepository for Repo {
    type Error = &'static str;
    fn store(&mut self, s: &ProductionSession) -> Result<(), Self::Error> { self.sessions.push(s.clone()); Ok(()) }
    fn update(&mut self, s: &ProductionSession) -> Result<(), Self::Error> { *self.sessions.iter_mut().find(|x| x.id == s.id).ok_or("session not found")? = s.clone(); Ok(()) }
    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> { Ok(self.sessions.iter().find(|s| &s.id == id).cloned()) }
}

struct State {
    repo: Repo,
    recording: Option<DistributedRecording>,
    persistence: InMemoryPersistenceProvider,
}

fn main() -> std::io::Result<()> {
    let mut repo = Repo { sessions: Vec::new() };
    let mut client = ClientSessionService::new(&mut repo);
    client.create(SESSION_ID, OWNER_ID).expect("session creation");
    drop(client);
    let listener = TcpListener::bind(ADDRESS)?;
    println!("NC-PoRe recording slice: http://{ADDRESS}/?actor=alice");
    println!("NC-PoRe recording slice: http://{ADDRESS}/?actor=bob");
    let mut state = State { repo, recording: None, persistence: InMemoryPersistenceProvider::new() };
    for stream in listener.incoming() { if let Ok(stream) = stream { handle(stream, &mut state); } }
    Ok(())
}

fn handle(mut stream: TcpStream, state: &mut State) {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 16 * 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                request.extend_from_slice(&chunk[..n]);
                if request.len() > MAX_REQUEST_BODY + 16 * 1024 { let _ = stream.write_all(response(json(413, "request_too_large")).as_bytes()); return; }
                if request_complete(&request) { break; }
            }
            Err(_) => return,
        }
    }
    let text = String::from_utf8_lossy(&request);
    let mut sections = text.splitn(2, "\r\n\r\n");
    let head = sections.next().unwrap_or_default();
    let body = sections.next().unwrap_or_default();
    let mut first = head.lines().next().unwrap_or_default().split_whitespace();
    let method = first.next().unwrap_or_default();
    let target = first.next().unwrap_or_default();
    let path = target.split_once('?').map_or(target, |(p, _)| p);
    let result = match (method, path) {
        ("GET", "/") => (200, "text/html; charset=utf-8", HTML.to_owned()),
        ("GET", p) if p.starts_with("/api/state") => state_json(state),
        ("POST", p) if p.ends_with("/join") => join(state, body),
        ("POST", p) if p.ends_with("/start") => start(state),
        ("POST", p) if p.ends_with("/recording/begin") => begin(state, body),
        ("POST", p) if p.ends_with("/recording/ready") => ready(state, body),
        ("POST", p) if p.ends_with("/recording/open") => open_recording(state, body),
        ("POST", p) if p.ends_with("/recording/opening-confirm") => opening_confirm(state, body),
        ("POST", p) if p.ends_with("/recording/stop") => stop(state, body),
        ("POST", p) if p.ends_with("/recording/stop-ack") => stop_ack(state, body),
        ("POST", p) if p.ends_with("/recording/finalize") => finalize_recording(state, body),
        ("POST", p) if p.ends_with("/recording/complete") => complete(state, body),
        _ => json(404, "not_found"),
    };
    let _ = stream.write_all(response(result).as_bytes());
}

fn request_complete(buf: &[u8]) -> bool {
    let Some(separator) = buf.windows(4).position(|w| w == b"\r\n\r\n") else { return false; };
    let head = String::from_utf8_lossy(&buf[..separator]);
    let length = head.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name.eq_ignore_ascii_case("content-length")).then(|| value.trim().parse::<usize>().ok())
    }).flatten().unwrap_or(0);
    buf.len() >= separator + 4 + length
}
fn response(result: (u16, &'static str, String)) -> String { format!("HTTP/1.1 {status} {reason}\r\nContent-Type: {ty}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", result.2.len(), status=result.0, reason=reason(result.0), ty=result.1, body=result.2) }
fn join(state: &mut State, body: &str) -> (u16, &'static str, String) {
    let Some(id) = field(body, "participant_id") else { return json(400, "invalid_request"); };
    let mut client = ClientSessionService::new(&mut state.repo);
    match client.add_participant(SESSION_ID, OWNER_ID, &id, [ClientRole::Participant]) { Ok(_) => json(200, "ok"), Err(ClientSessionError::ParticipantAlreadyExists) => json(409, "participant_already_exists"), Err(_) => json(409, "join_rejected") }
}
fn start(state: &mut State) -> (u16, &'static str, String) { let mut client = ClientSessionService::new(&mut state.repo); if client.start(SESSION_ID, OWNER_ID).is_ok() { json(200, "ok") } else { json(409, "start_rejected") } }
fn begin(state: &mut State, body: &str) -> (u16, &'static str, String) {
    let Some(actor) = field(body, "actor_id") else { return json(400, "invalid_request"); };
    let actor = ParticipantId::new(actor); if actor.value() != OWNER_ID { return json(403, "owner_required"); }
    if state.recording.is_some() { return json(409, "recording_already_active"); }
    let pid = ProductionId::new(SESSION_ID); let rid = nc_pore_core::recording::RecordingId::new(RECORDING_ID);
    let Some(mut session) = state.repo.get(&pid).ok().flatten() else { return json(404, "session_not_found"); };
    if !session.recordings().iter().any(|r| r.id() == &rid) { if session.add_recording_by(&actor, Recording::new(RECORDING_ID)).is_err() { return json(409, "recording_rejected"); } if state.repo.update(&session).is_err() { return json(500, "repository_error"); } }
    match begin_distributed_recording(&mut state.repo, &pid, &actor, &rid) { Ok(r) => { let out=recording_json(&r); state.recording=Some(r); (200,"application/json; charset=utf-8",out) }, Err(_) => json(409,"recording_begin_rejected") }
}
fn ready(state: &mut State, body: &str) -> (u16, &'static str, String) { let Some(id)=field(body,"participant_id") else{return json(400,"invalid_request")}; let p=ParticipantId::new(id); let Some(r)=state.recording.as_mut() else{return json(409,"recording_not_started")}; if mark_distributed_recording_ready(&mut state.repo,r,&p).is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"ready_rejected")} }
fn open_recording(state: &mut State, body: &str) -> (u16, &'static str, String) { if field(body,"actor_id").as_deref()!=Some(OWNER_ID){return json(403,"owner_required")}; let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")}; if r.trigger_opening().is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"opening_rejected")} }
fn opening_confirm(state: &mut State, body: &str) -> (u16, &'static str, String) { let Some(id)=field(body,"participant_id")else{return json(400,"invalid_request")}; let p=ParticipantId::new(id); let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")}; if confirm_distributed_recording_opening(&mut state.repo,r,&p).is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"opening_confirmation_rejected")} }
fn stop(state: &mut State, body: &str) -> (u16, &'static str, String) { let Some(id)=field(body,"actor_id")else{return json(400,"invalid_request")}; let a=ParticipantId::new(id); if a.value()!=OWNER_ID{return json(403,"owner_required")}; let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")}; let Some(mut s)=state.repo.get(&ProductionId::new(SESSION_ID)).ok().flatten()else{return json(404,"session_not_found")}; if s.stop_recording_by(&a,r.recording_id()).is_err(){return json(409,"stop_rejected")}; if state.repo.update(&s).is_err(){return json(500,"repository_error")}; let rid=r.recording_id().clone(); match reconstitute_distributed_recording(&state.repo,&ProductionId::new(SESSION_ID),&a,&rid){Ok(n)=>{*r=n;(200,"application/json; charset=utf-8",recording_json(r))},Err(_)=>json(409,"reconstitution_failed")} }
fn stop_ack(state: &mut State, body: &str) -> (u16, &'static str, String) { let Some(id)=field(body,"participant_id")else{return json(400,"invalid_request")}; let p=ParticipantId::new(id); let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")}; if acknowledge_distributed_recording_stop_in_core(&mut state.repo,r,&p).is_ok(){(200,"application/json; charset=utf-8",recording_json(r))}else{json(409,"stop_ack_rejected")} }
fn finalize_recording(state: &mut State, body: &str) -> (u16, &'static str, String) {
    let Some(actor)=field(body,"actor_id")else{return json(400,"invalid_request")}; if actor!=OWNER_ID{return json(403,"owner_required")};
    let Some(recording_id)=field(body,"recording_id")else{return json(400,"invalid_request")}; let Some(track_id)=field(body,"track_id")else{return json(400,"invalid_request")}; let Some(format)=field(body,"format")else{return json(400,"invalid_request")}; let Some(hex)=field(body,"payload_hex")else{return json(400,"invalid_request")}; let Ok(payload)=decode_hex(&hex)else{return json(400,"invalid_payload")};
    let handoff=BrowserRecordingHandoff::new(recording_id,track_id,payload,format,BrowserRecordingStopReason::UserRequested);
    match handoff.persist(RecordingSessionId::new(SESSION_ID),&mut state.persistence){Ok(a)=>json_data(200,&format!("{{\"artifact_id\":\"{}\",\"status\":\"{:?}\",\"payload_bytes\":{}}}",escape(a.id.value()),a.status(),a.tracks().iter().flat_map(|t|t.chunks()).map(|c|c.payload().size_bytes()).sum::<u64>())),Err(_)=>json(409,"artifact_persistence_rejected")}
}
fn complete(state: &mut State, body: &str) -> (u16, &'static str, String) { let Some(id)=field(body,"actor_id")else{return json(400,"invalid_request")}; let a=ParticipantId::new(id); if a.value()!=OWNER_ID{return json(403,"owner_required")}; let Some(r)=state.recording.as_mut()else{return json(409,"recording_not_started")}; let c=r.workflow().coordination(); if r.workflow().status()!=RecordingWorkflowStatus::Stopping||c.stop_acknowledged_participants().len()!=c.participants().len(){return json(409,"stop_ack_barrier_not_complete")}; let Some(mut s)=state.repo.get(&ProductionId::new(SESSION_ID)).ok().flatten()else{return json(404,"session_not_found")}; if s.complete_recording_by(&a,r.recording_id(),nc_pore_core::recording::RecordingArtifactId::new(ARTIFACT_ID)).is_err(){return json(409,"complete_rejected")}; if state.repo.update(&s).is_err(){return json(500,"repository_error")}; let rid=r.recording_id().clone(); match reconstitute_distributed_recording(&state.repo,&ProductionId::new(SESSION_ID),&a,&rid){Ok(n)=>{*r=n;(200,"application/json; charset=utf-8",recording_json(r))},Err(_)=>json(409,"reconstitution_failed")} }
fn state_json(state:&mut State)->(u16,&'static str,String){if let Some(r)=state.recording.as_ref(){(200,"application/json; charset=utf-8",recording_json(r))}else if let Some(s)=state.repo.get(&ProductionId::new(SESSION_ID)).ok().flatten(){let p=s.participations().iter().map(|p|format!("\"{}\"",escape(p.participant_id.value()))).collect::<Vec<_>>().join(",");(200,"application/json; charset=utf-8",format!("{{\"session_status\":\"{:?}\",\"participants\":[{p}],\"recording_status\":null,\"workflow\":null}}",s.status()))}else{json(200,"no_session")}}
fn recording_json(r:&DistributedRecording)->String{let c=r.workflow().coordination();let list=|ids:&[ParticipantId]|ids.iter().map(|p|format!("\"{}\"",escape(p.value()))).collect::<Vec<_>>().join(",");format!("{{\"recording_status\":\"{:?}\",\"workflow\":\"{:?}\",\"participants\":[{}],\"ready\":[{}],\"opening_confirmed\":[{}],\"stop_acknowledged\":[{}]}}",r.workflow().recording().status(),r.workflow().status(),list(c.participants()),list(c.ready_participants()),list(c.opening_confirmed_participants()),list(c.stop_acknowledged_participants()))}
fn field(body:&str,name:&str)->Option<String>{let marker=format!("\"{name}\":\"");let index=body.find(&marker)?+marker.len();let rest=&body[index..];Some(rest[..rest.find('"')?].to_owned())}
fn decode_hex(value:&str)->Result<Vec<u8>,()>{if value.len()%2!=0{return Err(())}value.as_bytes().chunks_exact(2).map(|p|Ok((hex_value(p[0])?<<4)|hex_value(p[1])?)).collect()}
fn hex_value(v:u8)->Result<u8,()>{match v{b'0'..=b'9'=>Ok(v-b'0'),b'a'..=b'f'=>Ok(v-b'a'+10),b'A'..=b'F'=>Ok(v-b'A'+10),_=Err(())}}
fn escape(v:&str)->String{v.replace('\\',"\\\\").replace('"',"\\\"")}
fn json(status:u16,error:&'static str)->(u16,&'static str,String){json_data(status,&format!("{{\"error\":\"{error}\"}}"))}
fn json_data(status:u16,body:&str)->(u16,&'static str,String){(status,"application/json; charset=utf-8",body.to_owned())}
fn reason(s:u16)->&'static str{match s{200=>"OK",400=>"Bad Request",403=>"Forbidden",404=>"Not Found",409=>"Conflict",413=>"Payload Too Large",500=>"Internal Server Error",_=>"Bad Request"}}

const HTML:&str=r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width"><title>NC-PoRe recording slice</title><style>body{font:16px system-ui;max-width:900px;margin:2rem auto;padding:0 1rem}button{margin:.25rem;padding:.5rem .75rem}.box{border:1px solid #aaa;padding:1rem;margin:1rem 0}.status{display:inline-flex;align-items:center;gap:.5rem;font-weight:700}.dot{width:1rem;height:1rem;border-radius:50%;display:inline-block;border:1px solid #555}.grey{background:#bbb}.black{background:#111}.red{background:#d00}.yellow{background:#ffd43b}.green{background:#2f9e44}.blue{background:#228be6}.confirmed{background:white;border:2px solid #2f9e44}.blink{animation:pulse 1s infinite}@keyframes pulse{50%{opacity:.35}}</style></head><body><h1>NC-PoRe recording client</h1><p>Open twice with <b>?actor=alice</b> and <b>?actor=bob</b>.</p><div class="box">Actor: <span id="actor"></span></div><div class="box"><div id="indicator" class="status"><span id="dot" class="dot grey"></span><span id="label">Session wird erstellt / vorbereitet</span></div></div><div class="box"><pre id="state">—</pre></div><div id="buttons"></div><script>const q=new URLSearchParams(location.search),actor=q.get('actor')||'bob',b=document.getElementById('buttons'),state=document.getElementById('state'),dot=document.getElementById('dot'),label=document.getElementById('label');let mediaRecorder=null,mediaStream=null,chunks=[],finalizePromise=Promise.resolve();document.getElementById('actor').textContent=actor;async function post(path,data){const r=await fetch(path,{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(data||{})});if(!r.ok)throw new Error((await r.json()).error);return r.json()}function button(text,path,data,disabled=false){const x=document.createElement('button');x.textContent=text;x.disabled=disabled;x.onclick=async()=>{try{if(path==='browser-start'){await startBrowserCapture();refresh();return}if(path==='browser-stop'){stopBrowserCapture();return}if(path==='/recording/stop'){stopBrowserCapture();await finalizePromise;await post(path,data);refresh();return}await post(path,data);refresh()}catch(e){alert(e)}};b.appendChild(x)}function status(workflow){dot.className='dot';if(workflow==='Preparing'){dot.classList.add('grey');label.textContent='Session wird erstellt / vorbereitet'}else if(workflow==='WaitingForReady'||workflow==='Ready'){dot.classList.add('yellow');label.textContent='Aufnahmebereit / READY'}else if(workflow==='Opening'){dot.classList.add('green','blink');label.textContent='Aufnahme wird gestartet'}else if(workflow==='Recording'){dot.classList.add('green');label.textContent='Aufnahme läuft'}else if(workflow==='Stopping'){dot.classList.add('blue');label.textContent='Aufnahme abgeschlossen / Übertragung läuft'}else if(workflow==='Completed'){dot.classList.add('confirmed');label.textContent='Serverseitig überprüft / übernommen'}else{dot.classList.add('grey');label.textContent='Kein aktiver Recording-Status'}}async function startBrowserCapture(){if(!navigator.mediaDevices?.getUserMedia||!window.MediaRecorder)throw new Error('Browser MediaRecorder nicht verfügbar');if(mediaRecorder&&mediaRecorder.state!=='inactive')return;mediaStream=await navigator.mediaDevices.getUserMedia({audio:true});chunks=[];let options={};if(MediaRecorder.isTypeSupported('audio/webm;codecs=opus'))options={mimeType:'audio/webm;codecs=opus'};else if(MediaRecorder.isTypeSupported('audio/webm'))options={mimeType:'audio/webm'};mediaRecorder=new MediaRecorder(mediaStream,options);mediaRecorder.ondataavailable=e=>{if(e.data.size)chunks.push(e.data)};mediaRecorder.onstop=()=>{finalizePromise=finalizeBrowserCapture().finally(()=>{mediaStream?.getTracks().forEach(t=>t.stop());mediaStream=null;mediaRecorder=null})};mediaRecorder.start(250)}async function finalizeBrowserCapture(){const blob=new Blob(chunks,{type:mediaRecorder?.mimeType||'audio/webm'}),bytes=new Uint8Array(await blob.arrayBuffer());let hex='';for(const byte of bytes)hex+=byte.toString(16).padStart(2,'0');await post('/recording/finalize',{actor_id:actor,recording_id:'vertical-slice-recording',track_id:'browser-track',format:blob.type||'audio/webm',payload_hex:hex})}function stopBrowserCapture(){if(mediaRecorder&&mediaRecorder.state!=='inactive')mediaRecorder.stop()}async function refresh(){let r=await(await fetch('/api/state')).json();b.replaceChildren();state.textContent=JSON.stringify(r,null,2);if(r.recording_status===null){status('Preparing');if(actor==='alice'){button('Start session','/start',{},r.session_status==='Active');if(r.session_status==='Active'&&r.participants.length>0)button('Begin recording','/recording/begin',{actor_id:actor})}button('Join','/join',{participant_id:actor},r.participants.includes(actor));return}status(r.workflow);if(r.workflow==='WaitingForReady'&&!r.ready.includes(actor))button('READY','/recording/ready',{participant_id:actor});if(actor==='alice'&&r.workflow==='Ready')button('Trigger Opening','/recording/open',{actor_id:actor});if(r.workflow==='Opening'&&!r.opening_confirmed.includes(actor))button('Confirm Opening','/recording/opening-confirm',{participant_id:actor});if(r.workflow==='Recording'){button(mediaRecorder&&mediaRecorder.state!=='inactive'?'Stop browser capture':'Start browser capture',mediaRecorder&&mediaRecorder.state!=='inactive'?'browser-stop':'browser-start');if(actor==='alice')button('Stop recording','/recording/stop',{actor_id:actor})}if(r.workflow==='Stopping'&&!r.stop_acknowledged.includes(actor))button('ACK stop','/recording/stop-ack',{participant_id:actor});if(actor==='alice'&&r.workflow==='Stopping'&&r.stop_acknowledged.length===r.participants.length)button('Complete recording','/recording/complete',{actor_id:actor})}setInterval(refresh,1000);refresh();</script></body></html>"#;
