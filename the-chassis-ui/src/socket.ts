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
  private url = 'ws://127.0.0.1:8765';

  constructor() {}

  connect(onMessage: (data: TelemetryData) => void, onConnectionChange: (connected: boolean) => void) {
    this.onMessageCallback = onMessage;
    this.onConnectionChange = onConnectionChange;
    this._connect();
  }

  private _connect() {
    console.log("Connecting to Telemetry...");
    try {
      this.socket = new WebSocket(this.url);
      
      this.socket.onopen = () => {
        console.log("Connected to The Chassis Telemetry");
        this.reconnecting = false;
        if (this.onConnectionChange) this.onConnectionChange(true);
      };

      this.socket.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (this.onMessageCallback) this.onMessageCallback(data);
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
