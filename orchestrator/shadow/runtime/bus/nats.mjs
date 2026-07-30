// NATS bus — optional. Graceful fallback to file-based if NATS not running.
// AHMAD-BOT publishes to arena.crawl.red
// EDUALC publishes to arena.crawl.blue
// BOB subscribes to both, reasons, publishes to arena.result

export async function tryConnect(url = 'nats://localhost:4222') {
  try {
    const { connect } = await import('nats');
    const nc = await connect({ servers: url, timeout: 3000 });
    return nc;
  } catch {
    return null;
  }
}

export async function publish(nc, subject, payload) {
  if (!nc) return false;
  await nc.publish(subject, new TextEncoder().encode(JSON.stringify(payload)));
  return true;
}

export async function subscribe(nc, subject, handler) {
  if (!nc) return;
  const sub = nc.subscribe(subject);
  for await (const msg of sub) {
    const data = JSON.parse(new TextDecoder().decode(msg.data));
    handler(data);
  }
}
