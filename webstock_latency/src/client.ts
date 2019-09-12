function randomStr(len: number) {
  const arr = 'abcdefghijklmnopqrstuvwxyz'.split('');
  var ans = '';
  for (var i = len; i > 0; i--) {
    ans += arr[Math.floor(Math.random() * arr.length)];
  }
  return ans;
}

function printHistogramData(data: number[]) {
  // Convert each to a delta-time
  const deltaTimes = [];
  for (let i = 1; i < data.length; i++) {
    deltaTimes.push(data[i] - data[i - 1]);
  }

  console.log(`Printing ${deltaTimes.length} deltas...`);

  // Group by rounded times
  const counts = {};
  for (const delta of deltaTimes) {
    if (delta in counts) {
      counts[delta] += 1;
    } else {
      counts[delta] = 1;
    }
  }

  (window as any).counts = counts;

  console.log('DeltaMs');
  for (const key of Object.keys(counts)) {
    console.log(key);
  }
  console.log('Counts');
  for (const key of Object.keys(counts)) {
    console.log(counts[key]);
  }
}

let packetTimes = [];

// const address = 'ws://www.friendzone-factorio.thilenius.org';
const address = 'ws://69.197.128.18';
const tickHz = 60;
const seconds = 60;

console.log('Connecting to:', address);

const ws = new WebSocket(address);
ws.onopen = event => {
  console.log(`Connected! Streaming at 100hz for ${tickHz * seconds} packets.`);
  ws.send('RESET');
  let packet = 0;

  const tick = () => {
    // Record time
    packetTimes.push(Math.round(performance.now()));

    // Send 1024 bytes of random data.
    console.log('Tick');
    ws.send(randomStr(1024));
    packet += 1;

    // Check if that was the last packet
    if (packet < tickHz * seconds) {
      requestAnimationFrame(tick);
    } else {
      ws.send('DUMP');
      printHistogramData(packetTimes);
    }
  };
  requestAnimationFrame(tick);
};

ws.onmessage = data => {};
