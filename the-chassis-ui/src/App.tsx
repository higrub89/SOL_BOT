import { useEffect, useState } from 'react';
import { Activity, AlertTriangle, Cpu, DollarSign, LayoutDashboard, Radio, Settings as SettingsIcon, Zap, FileText, TrendingUp } from 'lucide-react';
import { Area, AreaChart, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts';
import { socketClient } from './socket';
import './index.css';

// --- MAIN COMPONENTS ---

function TelemetryView({ data, connected, chartData }: any) {
  return (
    <div className="animate-slide-in">
      <header style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '2.5rem' }}>
        <div>
          <h1>Live Telemetry</h1>
          <p style={{ color: 'var(--text-muted)', marginTop: '0.5rem' }}>Institutional Grade Execution Layer</p>
        </div>
        
        <div className="status-indicator">
          <div className={`status-dot ${connected ? 'pulse-animation' : ''}`} style={{ background: connected ? 'var(--success)' : 'var(--danger)', boxShadow: `0 0 8px ${connected ? 'var(--success-glow)' : 'var(--danger-glow)'}` }} />
          {connected ? 'OPERATIONAL: ARMED' : 'DISCONNECTED'}
        </div>
      </header>

      {/* Top KPIs */}
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(240px, 1fr))', gap: '1.5rem' }}>
        <div className="glass-card stat-card" style={{ animationDelay: '0.1s' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
            <span className="stat-label">Vault Reserve</span>
            <DollarSign size={20} color="var(--primary)" />
          </div>
          <div className="stat-value primary">{data.balance.toFixed(5)} SOL</div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: 'auto' }}>Tier III • Critical Reserve</div>
        </div>

        <div className="glass-card stat-card" style={{ animationDelay: '0.2s' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
            <span className="stat-label">Net Yield (PNL)</span>
            <Activity size={20} color={data.pnl >= 0 ? 'var(--success)' : 'var(--danger)'} />
          </div>
          <div className={`stat-value ${data.pnl >= 0 ? 'success' : 'danger'}`}>
            {data.pnl > 0 ? '+' : ''}{data.pnl.toFixed(4)} SOL
          </div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: 'auto' }}>Cumulative Earnings</div>
        </div>

        <div className="glass-card stat-card" style={{ animationDelay: '0.3s' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
            <span className="stat-label">Engine Uptime</span>
            <Cpu size={20} color="var(--text-muted)" />
          </div>
          <div className="stat-value">{Math.floor(data.uptime / 60)}m {data.uptime % 60}s</div>
          <div style={{ fontSize: '0.8rem', color: 'var(--text-muted)', marginTop: 'auto' }}>High Performance Node</div>
        </div>
      </div>

      {/* Charts and Tables */}
      <div className="dashboard-grid-large" style={{ animationDelay: '0.4s' }}>
        <div className="glass-card">
          <h3 style={{ fontSize: '1.1rem', marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <TrendingUp size={18} color="var(--primary)" /> Yield Curve Diagnostics
          </h3>
          <div style={{ height: '300px', width: '100%' }}>
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={chartData} margin={{ top: 10, right: 10, left: -20, bottom: 0 }}>
                <defs>
                  <linearGradient id="colorPnl" x1="0" y1="0" x2="0" y2="1">
                    <stop offset="5%" stopColor="var(--primary)" stopOpacity={0.3} />
                    <stop offset="95%" stopColor="var(--primary)" stopOpacity={0} />
                  </linearGradient>
                </defs>
                <XAxis dataKey="time" stroke="var(--text-muted)" fontSize={12} tickLine={false} axisLine={false} />
                <YAxis stroke="var(--text-muted)" fontSize={12} tickLine={false} axisLine={false} tickFormatter={(value) => `${value} SOL`} />
                <Tooltip 
                  contentStyle={{ background: 'var(--bg-card)', border: '1px solid var(--glass-border)', borderRadius: '8px' }}
                  itemStyle={{ color: 'var(--primary)' }}
                />
                <Area type="monotone" dataKey="value" stroke="var(--primary)" strokeWidth={3} fillOpacity={1} fill="url(#colorPnl)" />
              </AreaChart>
            </ResponsiveContainer>
          </div>
        </div>

        <div className="glass-card">
          <h3 style={{ fontSize: '1.1rem', marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '8px' }}>
            <Radio size={18} color="var(--text-muted)" /> Active Allocations
          </h3>
          {data.positions.length === 0 ? (
            <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '250px', color: 'var(--text-muted)' }}>
              <span className="mono" style={{ fontSize: '0.9rem', opacity: 0.5 }}>NO ACTIVE ALLOCATIONS</span>
            </div>
          ) : (
            <div style={{ overflowX: 'auto' }}>
              <table>
                <thead>
                  <tr>
                    <th>Asset</th>
                    <th>Size</th>
                    <th>PNL</th>
                  </tr>
                </thead>
                <tbody>
                  {data.positions.map((pos: any, i: number) => (
                    <tr key={i}>
                      <td className="mono" style={{ color: 'var(--text-main)' }}>{pos.mint.substring(0, 6)}...</td>
                      <td className="mono">{pos.amount}</td>
                      <td className={`mono ${pos.pnl_pct >= 0 ? 'success' : 'danger'}`}>
                        {pos.pnl_pct > 0 ? '+' : ''}{pos.pnl_pct.toFixed(2)}%
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ExecutionLogView() {
  const dummyLogs = [
    { time: '15:16', action: 'DEGEN ENTRY', asset: 'Dze9...JyKZ', status: 'FAILED', reason: 'Jito Ix Error' },
    { time: '15:15', action: 'BUY', asset: '45nF...dRj', status: 'FAILED', reason: 'TOKEN_NOT_TRADABLE' },
    { time: '15:14', action: 'DEGEN ENTRY', asset: '45nF...dRj', status: 'FAILED', reason: 'Insufficient Safety Buffer' },
    { time: '15:12', action: 'PANIC_ALL', asset: 'ALL', status: 'SUCCESS', reason: 'User Initiated' },
    { time: '14:20', action: 'BUY', asset: 'FjR3...22yL', status: 'SUCCESS', reason: 'Strategy Triggered' }
  ];

  return (
    <div className="animate-slide-in">
      <header style={{ marginBottom: '2.5rem' }}>
        <h1>Execution Log</h1>
        <p style={{ color: 'var(--text-muted)', marginTop: '0.5rem' }}>Historical trade data and system events</p>
      </header>

      <div className="glass-card">
        <h3 style={{ fontSize: '1.1rem', marginBottom: '1.5rem', display: 'flex', alignItems: 'center', gap: '8px' }}>
          <FileText size={18} color="var(--primary)" /> Recent Operations
        </h3>
        
        <div style={{ overflowX: 'auto' }}>
          <table>
            <thead>
              <tr>
                <th>Time</th>
                <th>Action</th>
                <th>Asset</th>
                <th>Status</th>
                <th>Details</th>
              </tr>
            </thead>
            <tbody>
              {dummyLogs.map((log, i) => (
                <tr key={i}>
                  <td className="mono" style={{ color: 'var(--text-muted)' }}>{log.time}</td>
                  <td className="mono" style={{ color: 'var(--primary)' }}>{log.action}</td>
                  <td className="mono">{log.asset}</td>
                  <td>
                    <span className={`status-indicator ${log.status === 'SUCCESS' ? 'success' : 'danger'}`} style={{ padding: '0.25rem 0.5rem', fontSize: '0.75rem', background: log.status === 'SUCCESS' ? 'rgba(0, 255, 157, 0.1)' : 'rgba(255, 42, 95, 0.1)', color: log.status === 'SUCCESS' ? 'var(--success)' : 'var(--danger)', border: log.status === 'SUCCESS' ? '1px solid rgba(0, 255, 157, 0.2)' : '1px solid rgba(255, 42, 95, 0.2)'}}>
                      {log.status}
                    </span>
                  </td>
                  <td className="mono" style={{ fontSize: '0.85rem', color: 'var(--text-muted)' }}>{log.reason}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

function SettingsView() {
  return (
    <div className="animate-slide-in">
      <header style={{ marginBottom: '2.5rem' }}>
        <h1>System Config</h1>
        <p style={{ color: 'var(--text-muted)', marginTop: '0.5rem' }}>Engine parameters and security tolerances</p>
      </header>

      <div className="dashboard-grid">
        <div className="glass-card">
          <h3 style={{ fontSize: '1.1rem', marginBottom: '1.5rem', color: 'var(--primary)' }}>Jito MEV Engine</h3>
          
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
            <div>
              <label style={{ display: 'block', color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '0.5rem' }}>Bundle Tip Priority (Lamports)</label>
              <input type="text" value="100,000" disabled style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.3)', border: '1px solid var(--glass-border)', color: 'white', borderRadius: '4px', fontFamily: 'var(--font-mono)' }} />
              <p style={{ fontSize: '0.75rem', color: 'var(--text-muted)', marginTop: '0.25rem' }}>Fixed in core executor (executor_v2.rs)</p>
            </div>
            
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '1rem', background: 'rgba(0, 255, 157, 0.05)', borderRadius: '8px', border: '1px solid rgba(0, 255, 157, 0.1)' }}>
              <div>
                <strong style={{ color: 'var(--success)', display: 'block' }}>MEV Protection</strong>
                <span style={{ fontSize: '0.8rem', color: 'var(--text-muted)' }}>Atomic Sandwhich Attack Defense</span>
              </div>
              <div style={{ background: 'var(--success)', color: '#000', padding: '0.2rem 0.5rem', borderRadius: '4px', fontSize: '0.75rem', fontWeight: 700 }}>ARMED</div>
            </div>
          </div>
        </div>

        <div className="glass-card">
          <h3 style={{ fontSize: '1.1rem', marginBottom: '1.5rem', color: 'var(--primary)' }}>Security Guards</h3>
          
          <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
             <div>
              <label style={{ display: 'block', color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '0.5rem' }}>Vault Safety Buffer</label>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <input type="text" value="0.015" disabled style={{ flex: 1, padding: '0.75rem', background: 'rgba(0,0,0,0.3)', border: '1px solid var(--glass-border)', color: 'white', borderRadius: '4px', fontFamily: 'var(--font-mono)' }} />
                <span style={{ color: 'var(--text-muted)', fontWeight: 600 }}>SOL</span>
              </div>
            </div>

             <div>
              <label style={{ display: 'block', color: 'var(--text-muted)', fontSize: '0.85rem', marginBottom: '0.5rem' }}>Minimum Pool Liquidity</label>
              <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                <input type="text" value="$ 500.00" disabled style={{ flex: 1, padding: '0.75rem', background: 'rgba(0,0,0,0.3)', border: '1px solid var(--glass-border)', color: 'white', borderRadius: '4px', fontFamily: 'var(--font-mono)' }} />
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function App() {
  const [activeTab, setActiveTab] = useState('telemetry');
  const [data, setData] = useState({
    positions: [] as any[],
    balance: 0.041398,
    uptime: 326, // 5h 26m
    pnl: -0.107348,
    last_updates: [] as any[]
  });
  
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    socketClient.connect((telemetry) => {
      setData(prev => ({ ...prev, ...telemetry }));
    }, (status) => setConnected(status));
  }, []);

  const handlePanicAll = () => {
    if (confirm("⚠️ ENGINES ON: ARE YOU SURE YOU WANT TO PANIC SELL ALL POSITIONS?")) {
      socketClient.sendPanicAll();
    }
  };

  const chartData = [
    { time: '10:00', value: data.pnl - 0.05 },
    { time: '10:15', value: data.pnl - 0.02 },
    { time: '10:30', value: data.pnl - 0.08 },
    { time: '10:45', value: data.pnl + 0.01 },
    { time: 'LIVE', value: data.pnl },
  ];

  const renderContent = () => {
    switch(activeTab) {
      case 'telemetry': return <TelemetryView data={data} connected={connected} chartData={chartData} />;
      case 'log': return <ExecutionLogView />;
      case 'settings': return <SettingsView />;
      default: return <TelemetryView data={data} connected={connected} chartData={chartData} />;
    }
  };

  const getTabStyle = (tabId: string) => {
    const isActive = activeTab === tabId;
    return { 
      display: 'flex', 
      alignItems: 'center', 
      gap: '12px', 
      padding: '12px 16px', 
      borderRadius: '8px', 
      color: isActive ? 'var(--primary)' : 'var(--text-muted)', 
      background: isActive ? 'rgba(0, 240, 255, 0.05)' : 'transparent', 
      textDecoration: 'none', 
      fontWeight: isActive ? 600 as const : 500 as const,
      cursor: 'pointer',
      transition: 'all 0.2s',
      borderLeft: isActive ? '3px solid var(--primary)' : '3px solid transparent'
    };
  };

  return (
    <div className="app-container">
      {/* Sidebar Navigation */}
      <aside className="sidebar">
        <div style={{ paddingBottom: '2rem', borderBottom: '1px solid rgba(255,255,255,0.05)' }}>
          <h2 style={{ display: 'flex', alignItems: 'center', gap: '8px', fontSize: '1.25rem' }}>
            <Zap className="primary" style={{ color: 'var(--primary)', filter: 'drop-shadow(0 0 8px var(--primary-glow))' }} />
            <span style={{ fontWeight: 700, letterSpacing: '0.1em' }}>THE CHASSIS</span>
          </h2>
          <p style={{ color: 'var(--text-muted)', fontSize: '0.75rem', marginTop: '0.5rem', fontFamily: 'var(--font-mono)' }}>
            v2.2 • S-CLASS
          </p>
        </div>

        <nav style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
          <div onClick={() => setActiveTab('telemetry')} style={getTabStyle('telemetry')}>
            <LayoutDashboard size={20} />
            Telemetry
          </div>
          <div onClick={() => setActiveTab('log')} style={getTabStyle('log')}>
            <Activity size={20} />
            Execution Log
          </div>
          <div onClick={() => setActiveTab('settings')} style={getTabStyle('settings')}>
            <SettingsIcon size={20} />
            Settings
          </div>
        </nav>

        <div style={{ marginTop: 'auto', padding: '1.5rem', background: 'rgba(255,42,95,0.05)', borderRadius: '12px', border: '1px solid rgba(255,42,95,0.2)' }}>
          <h4 style={{ color: 'var(--danger)', fontSize: '0.85rem', marginBottom: '1rem', display: 'flex', alignItems: 'center', gap: '8px', textTransform: 'uppercase', letterSpacing: '0.05em' }}>
            <AlertTriangle size={16} /> EMERGENCY
          </h4>
          <button className="btn btn-danger" onClick={handlePanicAll} style={{ width: '100%', justifyContent: 'center' }}>
            PANIC ALL
          </button>
        </div>
      </aside>

      {/* Main Container */}
      <main className="main-content" style={{ position: 'relative' }}>
         {renderContent()}
      </main>
    </div>
  );
}

export default App;
