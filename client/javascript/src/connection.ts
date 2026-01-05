import { Conn, Message as CapnpMessage } from "capnp-es";
import { Message as RpcMessage } from "capnp-es/capnp/rpc";
import net from "node:net";
import { TaxEngine } from "./capnp/references.js";
import type { TaxEngine$Client } from "./capnp/references.js";

class TcpTransport {
  private socket: net.Socket;
  private buffered = Buffer.alloc(0);
  private pending: Array<(msg: RpcMessage) => void> = [];
  private queued: RpcMessage[] = [];
  private ended = false;

  constructor(socket: net.Socket) {
    this.socket = socket;
    this.socket.on("data", (chunk) => this.onData(chunk));
    this.socket.on("end", () => this.onEnd());
    this.socket.on("close", () => this.onEnd());
    this.socket.on("error", () => this.onEnd());
  }

  sendMessage(msg: RpcMessage): void {
    const message = msg.segment.message;
    const buf = Buffer.from(new Uint8Array(message.toArrayBuffer()));
    this.socket.write(buf);
  }

  recvMessage(): Promise<RpcMessage> {
    const existing = this.queued.shift();
    if (existing) {
      return Promise.resolve(existing);
    }
    if (this.ended) {
      return Promise.reject();
    }
    return new Promise<RpcMessage>((resolve) => {
      this.pending.push(resolve);
    });
  }

  close(): void {
    this.ended = true;
    this.socket.end();
    this.socket.destroy();
    this.pending = [];
    this.queued = [];
  }

  private onEnd(): void {
    this.ended = true;
    this.pending = [];
    this.queued = [];
  }

  private onData(chunk: Buffer | string): void {
    if (this.ended) return;
    const buf = typeof chunk === "string" ? Buffer.from(chunk) : chunk;
    this.buffered = Buffer.concat([this.buffered, buf]);

    for (;;) {
      const frame = this.tryReadFrame();
      if (!frame) break;
      const msg = frame.getRoot(RpcMessage);

      const waiter = this.pending.shift();
      if (waiter) {
        waiter(msg);
      } else {
        this.queued.push(msg);
      }
    }
  }

  private tryReadFrame(): CapnpMessage | null {
    // Unpacked stream framing (matches capnp_futures::serialize::try_read_message used by the Rust server).
    if (this.buffered.length < 4) return null;

    const segmentCount = this.buffered.readUInt32LE(0) + 1;
    const headerBytes = 4 + segmentCount * 4;
    const alignedHeaderBytes = headerBytes + (headerBytes % 8);

    if (this.buffered.length < alignedHeaderBytes) return null;

    let segmentsBytes = 0;
    for (let i = 0; i < segmentCount; i++) {
      const words = this.buffered.readUInt32LE(4 + i * 4);
      segmentsBytes += words * 8;
    }

    const totalBytes = alignedHeaderBytes + segmentsBytes;
    if (this.buffered.length < totalBytes) return null;

    const frameBytes = this.buffered.subarray(0, totalBytes);
    this.buffered = this.buffered.subarray(totalBytes);

    // `packed=false` because the Rust side uses capnp::serialize (unpacked).
    return new CapnpMessage(frameBytes, false);
  }
}

function parseHostPort(addr: string): { host: string; port: number } {
  const [host, portStr] = addr.split(":");
  const port = Number(portStr);
  if (!host || !Number.isFinite(port)) {
    throw new Error(`Invalid address: ${addr}`);
  }
  return { host, port };
}

export async function connectTaxEngine(addr: string) {
  const { host, port } = parseHostPort(addr);

  const socket = net.createConnection({ host, port });
  socket.setNoDelay(true);

  await new Promise<void>((resolve, reject) => {
    socket.once("connect", resolve);
    socket.once("error", reject);
  });

  const transport = new TcpTransport(socket);
  const conn = new Conn(transport);
  const client = conn.bootstrap(TaxEngine) as TaxEngine$Client;

  return {
    client,
    close: () => {
      conn.shutdown();
      transport.close();
    },
  };
}