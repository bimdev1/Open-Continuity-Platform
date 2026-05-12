import React, { useState, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'motion/react';
import { 
  Shield, 
  Network, 
  Terminal as TerminalIcon, 
  Smartphone, 
  Monitor, 
  Clipboard, 
  Lock, 
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  FileCode,
  Info
} from 'lucide-react';

interface LogEntry {
  id: number;
  timestamp: string;
  source: 'LINUX' | 'ANDROID' | 'SYSTEM';
  message: string;
  type: 'info' | 'success' | 'warning' | 'error';
}

export default function App() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [isSimulating, setIsSimulating] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);

  const addLog = (source: LogEntry['source'], message: string, type: LogEntry['type'] = 'info') => {
    setLogs(prev => [...prev, {
      id: Date.now() + Math.random(),
      timestamp: new Date().toLocaleTimeString(),
      source,
      message,
      type
    }].slice(-50));
  };

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs]);

  const runSimulation = async () => {
    if (isSimulating) return;
    setIsSimulating(true);
    setLogs([]);

    addLog('SYSTEM', 'Initializing Open Continuity Platform MVP...');
    await sleep(800);
    addLog('LINUX', 'ocad: Registering D-Bus name org.oca.ocad...');
    await sleep(600);
    addLog('LINUX', 'ocad: Service active at /org/oca/ocad', 'success');
    
    await sleep(800);
    addLog('ANDROID', 'OcaService: Acquiring WiFi MulticastLock...');
    await sleep(400);
    addLog('ANDROID', 'OcaService: Starting foreground service...', 'success');
    
    await sleep(1000);
    addLog('SYSTEM', 'mDNS: Discovery started on _oca._tcp.local');
    await sleep(1200);
    addLog('ANDROID', 'mDNS: Peer "ocad-linux-host" discovered at 192.168.1.15:5005', 'success');
    
    await sleep(1000);
    addLog('SYSTEM', 'Handshake: Initiating Ed25519 mutual authentication...');
    await sleep(800);
    addLog('ANDROID', 'Handshake: Sending Public Key [25...eA]');
    await sleep(600);
    addLog('LINUX', 'Handshake: Received peer Public Key. Validating...', 'info');
    await sleep(400);
    addLog('LINUX', 'Handshake: Identity verified. Sending ACK + Public Key', 'success');
    
    await sleep(1000);
    addLog('SYSTEM', 'Session: ChaCha20-Poly1305 derived secret active.', 'success');
    
    await sleep(1500);
    addLog('LINUX', 'DBus: Method SendClipboard("Hello from Linux!") invoked.');
    addLog('LINUX', 'ocad: Encrypting payload...');
    await sleep(600);
    addLog('SYSTEM', 'TCP: Transmitting ClipboardPayload (V1, Type 0x02, Len 64)');
    
    await sleep(800);
    addLog('ANDROID', 'TCP: Received encrypted payload. Verifying tag...', 'info');
    await sleep(500);
    addLog('ANDROID', 'OcaService: Decrypted: "Hello from Linux!"', 'success');
    addLog('ANDROID', 'OcaService: Propagating to system clipboard.');

    setIsSimulating(false);
  };

  const sleep = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

  return (
    <div className="h-screen w-full flex flex-col font-sans bg-[#f8fafc] overflow-hidden border-8 border-slate-100">
      {/* Header */}
      <header className="h-16 bg-slate-900 text-white flex items-center justify-between px-8 shrink-0 shadow-lg z-10">
        <div className="flex items-center space-x-3">
          <div className="w-8 h-8 bg-blue-500 rounded flex items-center justify-center font-bold text-lg italic">O</div>
          <h1 className="font-semibold tracking-tight text-xl">
            Open Continuity API <span className="text-blue-400 font-light opacity-80 italic text-sm">/ v0.1.0-alpha</span>
          </h1>
        </div>
        
        <div className="flex space-x-6">
          <div className="flex space-x-6 text-sm font-mono items-center">
            <div className="flex items-center space-x-2">
              <span className={`w-2 h-2 rounded-full ${isSimulating ? 'bg-emerald-400 animate-pulse' : 'bg-gray-600'}`}></span>
              <span className="text-xs uppercase tracking-tighter">CORE: {isSimulating ? 'ACTIVE' : 'READY'}</span>
            </div>
            <div className="flex items-center space-x-2">
              <span className={`w-2 h-2 rounded-full ${isSimulating ? 'bg-emerald-400 animate-pulse' : 'bg-gray-600'}`}></span>
              <span className="text-xs uppercase tracking-tighter">MDNS: {isSimulating ? 'BROADCASTING' : 'IDLE'}</span>
            </div>
          </div>
          
          <button 
            onClick={runSimulation}
            disabled={isSimulating}
            className={`flex items-center gap-2 px-6 py-1.5 rounded-md font-bold transition-all text-sm border ${
              isSimulating 
                ? 'bg-slate-800 border-slate-700 text-slate-500' 
                : 'bg-blue-600 border-blue-500 text-white hover:bg-blue-500 hover:shadow-lg'
            }`}
          >
            {isSimulating ? <RefreshCw className="animate-spin" size={14} /> : <TerminalIcon size={14} />}
            {isSimulating ? 'SIMULATING...' : 'INTEGRATION TEST'}
          </button>
        </div>
      </header>

      {/* Main Grid */}
      <main className="flex-1 grid grid-cols-12 gap-0 overflow-hidden">
        {/* Left Sidebar: Architecture */}
        <section className="col-span-3 border-r border-slate-200 bg-white flex flex-col p-6 space-y-6">
          <h2 className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-2">System Node Architecture</h2>
          <div className="space-y-4 overflow-y-auto">
            <div className="p-4 border border-slate-100 rounded-lg bg-slate-50 hover:border-blue-200 transition-colors">
              <div className="flex justify-between items-start mb-2">
                <span className="font-mono text-sm font-bold text-slate-700">liboca (Shared)</span>
                <span className="text-[10px] px-2 py-0.5 bg-emerald-100 text-emerald-700 rounded font-bold uppercase tracking-tighter">RUST</span>
              </div>
              <p className="text-[11px] text-slate-500 leading-tight">Shared binary core handling Ed25519 keypairs, AEAD encryption, and mDNS-SD discovery.</p>
            </div>

            <div className="p-4 border border-slate-100 rounded-lg bg-slate-50 hover:border-blue-200 transition-colors">
              <div className="flex justify-between items-start mb-2">
                <span className="font-mono text-sm font-bold text-slate-700">ocad (Linux)</span>
                <span className="text-[10px] px-2 py-0.5 bg-blue-100 text-blue-700 rounded font-bold uppercase tracking-tighter">ZBUS</span>
              </div>
              <p className="text-[11px] text-slate-500 leading-tight">Systemd user service exposing a D-Bus interface for desktop integration.</p>
            </div>

            <div className="p-4 border border-slate-100 rounded-lg bg-slate-50 hover:border-blue-200 transition-colors">
              <div className="flex justify-between items-start mb-2">
                <span className="font-mono text-sm font-bold text-slate-700">Android JNI</span>
                <span className="text-[10px] px-2 py-0.5 bg-purple-100 text-purple-700 rounded font-bold uppercase tracking-tighter">KOTLIN</span>
              </div>
              <p className="text-[11px] text-slate-500 leading-tight">Kotlin foreground service managing liboca lifecycle via Rust bridge.</p>
            </div>
          </div>

          <div className="mt-auto p-4 bg-slate-900 rounded-lg text-[10px] font-mono text-blue-300">
            <div className="opacity-50">LOCAL_KEY_FINGERPRINT:</div>
            <div className="text-white break-all mt-1 font-bold">ed25519:6f7a_2b91_c0e8_4d22_90e1_ac3a_bb42</div>
          </div>
        </section>

        {/* Center Section: Activity & Peers */}
        <section className="col-span-6 bg-[#f1f5f9] flex flex-col overflow-hidden">
          <div className="p-6 border-b border-slate-200 bg-white/50">
            <h2 className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-4">Active mDNS Peers Discovery</h2>
            <div className="grid grid-cols-1 gap-2">
              <div className="flex items-center justify-between p-3 bg-white rounded border border-slate-200 shadow-sm transition-all hover:shadow-md">
                <div className="flex items-center space-x-3">
                  <div className={`w-2.5 h-2.5 rounded-full ${isSimulating ? 'bg-emerald-500' : 'bg-slate-300'}`}></div>
                  <div>
                    <div className="font-mono text-sm font-bold text-slate-700">pixel-7-pro.local</div>
                    <div className="text-[10px] text-slate-400 font-mono">192.168.1.104:5005</div>
                  </div>
                </div>
                <div className={`text-[10px] font-bold px-3 py-1 rounded border ${
                   isSimulating ? 'text-blue-600 border-blue-200 bg-blue-50' : 'text-slate-400 border-slate-100 bg-slate-50'
                }`}>
                  {isSimulating ? 'ESTABLISHED' : 'STANDBY'}
                </div>
              </div>
              
              <div className="flex items-center justify-between p-3 bg-white rounded border border-slate-200 shadow-sm opacity-60 grayscale">
                <div className="flex items-center space-x-3">
                  <div className="w-2.5 h-2.5 bg-amber-500 rounded-full"></div>
                  <div>
                    <div className="font-mono text-sm font-bold text-slate-700">thinkpad-x1.local</div>
                    <div className="text-[10px] text-slate-400 font-mono">192.168.1.112:5005</div>
                  </div>
                </div>
                <div className="text-[10px] font-bold text-slate-400 border border-slate-200 px-3 py-1 rounded">AUTH_PENDING</div>
              </div>
            </div>
          </div>

          <div className="flex-1 p-6 overflow-hidden flex flex-col">
            <h2 className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-4 flex items-center justify-between">
              <span>Clipboard Activity Stream</span>
              <span className="font-mono text-[10px] opacity-60">AES-GCM / ChaCha20-Poly1305</span>
            </h2>
            
            <div 
              ref={scrollRef}
              className="flex-1 bg-slate-950 rounded-lg border border-slate-800 p-4 font-mono text-[11px] leading-relaxed text-slate-300 space-y-1 overflow-y-auto"
            >
              {logs.length === 0 ? (
                <div className="h-full flex items-center justify-center text-slate-600 italic">
                  Waiting for network activity...
                </div>
              ) : (
                logs.map((log) => (
                  <motion.div 
                    key={log.id}
                    initial={{ opacity: 0, x: -5 }}
                    animate={{ opacity: 1, x: 0 }}
                    className="flex gap-2"
                  >
                    <span className="text-slate-600 shrink-0">[{log.timestamp}]</span>
                    <span className={`${
                      log.type === 'success' ? 'text-emerald-400 font-bold' :
                      log.type === 'error' ? 'text-rose-400' :
                      log.type === 'warning' ? 'text-amber-400' :
                      log.type === 'info' && log.source === 'SYSTEM' ? 'text-blue-400' :
                      'text-slate-400'
                    }`}>
                      {log.source}: {log.message}
                    </span>
                  </motion.div>
                ))
              )}
            </div>
          </div>
        </section>

        {/* Right Sidebar: Protocol */}
        <section className="col-span-3 border-l border-slate-200 bg-white flex flex-col p-6 overflow-hidden">
          <h2 className="text-xs font-bold text-slate-400 uppercase tracking-widest mb-4">Protocol Visualizer</h2>
          
          <div className="bg-slate-50 border border-slate-200 rounded p-4 mb-6">
            <div className="text-[10px] text-slate-400 mb-3 uppercase tracking-wider font-bold">ClipboardPayload Layout</div>
            <div className="grid grid-cols-4 gap-1 text-[10px] font-mono text-center">
              <div className="bg-blue-600 text-white py-2 rounded-sm font-bold" title="Version">VER</div>
              <div className="bg-indigo-600 text-white py-2 rounded-sm font-bold" title="Type">TYPE</div>
              <div className="bg-slate-800 text-white py-2 col-span-2 rounded-sm font-bold" title="Length">LENGTH (32)</div>
              
              <div className="bg-slate-200 p-3 col-span-4 rounded-sm mt-1 h-32 flex flex-wrap content-start overflow-hidden text-[#1e293b] leading-tight break-all border border-slate-300">
                <span className="opacity-40">01 02 00 00 </span>
                <span>AF 42 C0 91 DC ED 44 BB ... 42 FE 11 9A FF AA 91 32 44 88 11 00 EE 99 22 11 00 EE 99 22 11 00 EE 99 22 11 00 EE 99 22</span>
                <span className="text-blue-600 mt-2 block w-full text-left italic">[ENCRYPTED_DATA_BUFFER]</span>
              </div>
              
              <div className="bg-rose-600 text-white py-2 col-span-4 rounded-sm mt-1 font-bold shadow-sm" title="Poly1305 Tag">AUTH_TAG (16-byte HMAC)</div>
            </div>
          </div>

          <div className="space-y-4 text-sm">
            {[
              ['Encryption', 'ChaCha20'],
              ['Hash/MAC', 'Poly1305'],
              ['Key Exchange', 'X25519'],
              ['Identity', 'Ed25519'],
              ['MTU Limit', '1400 bytes'],
            ].map(([label, val]) => (
              <div key={label} className="flex justify-between items-center border-b border-slate-100 pb-2">
                <span className="text-xs text-slate-500 font-medium">{label}</span>
                <span className="text-xs font-mono font-bold text-slate-700 bg-slate-100 px-2 py-0.5 rounded">{val}</span>
              </div>
            ))}
          </div>

          <div className="mt-auto pt-6 group">
            <div className="w-full h-24 bg-slate-100 rounded-lg flex items-center justify-center border-2 border-dashed border-slate-200 group-hover:bg-slate-50 group-hover:border-blue-200 transition-all">
              <Shield className="w-10 h-10 text-slate-300 group-hover:text-blue-300 transition-colors" />
            </div>
            <p className="text-[10px] text-center text-slate-400 mt-3 flex items-center justify-center gap-1">
              <CheckCircle2 size={10} className="text-emerald-500" />
              Hardware acceleration: AVX2/NEON enabled
            </p>
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer className="h-12 bg-white border-t border-slate-200 flex items-center px-8 justify-between shrink-0 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.05)]">
        <div className="text-[10px] font-bold text-slate-400 uppercase tracking-[0.2em] flex items-center gap-2">
          <Info size={12} className="text-blue-500" />
          Internal Diagnostics Layer v1.0.42-alpha
        </div>
        <div className="flex items-center space-x-6 text-[10px] font-mono text-slate-500 font-bold uppercase">
          <div className="flex items-center gap-1.5">
            <span className="w-1.5 h-1.5 bg-emerald-500 rounded-full"></span>
            UPTIME: 04:12:45
          </div>
          <div className="flex items-center gap-1.5 ">
             CPU: <span className="text-slate-800">1.2%</span>
          </div>
          <div className="flex items-center gap-1.5">
             MEM: <span className="text-slate-800">14.2 MB</span>
          </div>
        </div>
      </footer>
    </div>
  );
}
