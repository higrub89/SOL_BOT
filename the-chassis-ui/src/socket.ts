type TelemetryData = {
  positions: any[];
  balance: number;
  uptime: number;
  pnl: number;
  last_updates: any[];
};

export class TelemetryClient {
  private socket: WebSocket | null = null;
  private onMessageCallback: ((data: TelemetryData) => void) | null = null;
  private onConnectionChange: ((connected: boolean) => void) | null = null;
  private reconnecting = false;
  private connectionStart: number | null = null;
  private url = 'ws://127.0.0.1:9001';

  constructor() {}

  connect(onMessage: (data: TelemetryData) => void, onConnectionChange: (connected: boolean) => void) {
    this.onMessageCallback = onMessage;
    this.onConnectionChange = onConnectionChange;
    this.connectionStart = Date.now();
    this._connect();
  }

  private _connect() {
    console.log("Connecting to Telemetry...");
    try {
      this.socket = new WebSocket(this.url);
      
      this.socket.onopen = () => {
        console.log("Connected to The Chassis Telemetry");
        this.reconnecting = false;
        this.connectionStart = Date.now();
        if (this.onConnectionChange) this.onConnectionChange(true);
      };

      this.socket.onmessage = (event) => {
        try {
          const raw = JSON.parse(event.data);
          const mappedData = {
            positions: raw.positions ? raw.positions.map((p: any) => ({
              mint: p.mint,
              symbol: p.symbol,
              amount: 'LIVE', 
              pnl_pct: p.yield_pct
            })) : [],
            balance: typeof raw.wallet_balance === 'number' ? raw.wallet_balance : 0,
            uptime: this.connectionStart ? Math.floor((Date.now() - this.connectionStart) / 1000) : 0,
            pnl: typeof raw.net_pnl === 'number' ? raw.net_pnl : 0,
            last_updates: []
          };
          if (this.onMessageCallback) this.onMessageCallback(mappedData as any);
        } catch (err) {
          console.error("Payload Parse Error", err);
        }
      };

      this.socket.onclose = () => {
        if (this.onConnectionChange) this.onConnectionChange(false);
        this.handleReconnect();
      };

      this.socket.onerror = (err) => {
        console.error("WebSocket Error:", err);
      };

    } catch (err) {
      console.error("Socket Instantiation Error:", err);
      this.handleReconnect();
    }
  }

  private handleReconnect() {
    if (this.reconnecting) return;
    this.reconnecting = true;
    console.log("Reconnecting in 2 seconds...");
    setTimeout(() => {
      this._connect();
    }, 2000);
  }

  // Panic All Command directly via Websocket
  sendPanicAll() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({
        command: "PANIC_ALL",
        timestamp: Date.now()
      }));
      console.log("🚨 PANIC ALL COMMAND SENT");
      return true;
    }
    return false;
  }
}

export const socketClient = new TelemetryClient();
