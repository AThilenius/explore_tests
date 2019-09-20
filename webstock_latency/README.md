# Websocket Latency Variance Test

![Results Histogram](readme_assets/histogram.png?raw=true)

This test fires off 1024B packets at 60HZ from a web browser via a WebSocket.
The 60HZ timer is done via `requestAnimationFrame` which gives fairly
consistent timing.

The happy-path was found to be 'acceptable' for real-time game networking, with
a decent distribution. The tail is still 5.3X the tick rate delta though, which
is still problematic. And this is over a strong WiFi connection and high speed
internet to a dedicated server.

TL;DR from experiment: Use UDP for state-sync game networking, even today. Sadly
UDP isn't available to web, apart from WebRTC which is a monster.
