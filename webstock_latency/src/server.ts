import WebSocket from 'ws';

function nowMillisecondsRounded(): number {
  const hrTime = process.hrtime();
  return Math.round(hrTime[0] * 1000 + hrTime[1] / 1000000);
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

  console.log('DeltaMs\tCount');
  for (const key of Object.keys(counts)) {
    console.log(`${key}\t${counts[key]}`);
  }
}

const wss = new WebSocket.Server({ port: 80 });
let packetTimes = [];

console.log('Waiting for connection..');
wss.on('connection', ws => {
  console.log('Client connected!');
  ws.on('message', message => {
    if (message === 'RESET') {
      packetTimes = [];
    } else if (message === 'DUMP') {
      printHistogramData(packetTimes);
      // ws.send(histogram);
      packetTimes = [];
    } else {
      packetTimes.push(nowMillisecondsRounded());
    }
  });
});
