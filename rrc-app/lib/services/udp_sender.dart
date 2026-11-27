import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

class UdpSender {
  static const host = '192.168.4.1';
  static const port = 8888;
  static const _interval = Duration(milliseconds: 10);

  /// ASCII "RC", big-endian
  static const _magic = [0x52, 0x43];

  /// magic (2) + throttle (1) + steering (1)
  static const _packetSize = 4;

  static const _min = -100;
  static const _max = 100;

  RawDatagramSocket? _socket;
  Timer? _timer;

  int throttle = 0;
  int steering = 0;

  Future<void> start() async {
    _socket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, 0);
    _timer = Timer.periodic(_interval, (_) {
      final data = Int8List(_packetSize)
        ..[0] = _magic[0]
        ..[1] = _magic[1]
        ..[2] = throttle.clamp(_min, _max)
        ..[3] = steering.clamp(_min, _max);
      _socket?.send(data, InternetAddress(host), port);
    });
  }

  void dispose() {
    _timer?.cancel();
    _socket?.close();
  }
}
