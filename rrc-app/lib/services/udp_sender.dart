import 'dart:async';
import 'dart:io';
import 'dart:typed_data';

class UdpSender {
  static const host = '192.168.4.1';
  static const port = 8888;
  static const _interval = Duration(milliseconds: 10);

  RawDatagramSocket? _socket;
  Timer? _timer;

  int throttle = 0;
  int steering = 0;

  Future<void> start() async {
    _socket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, 0);
    _timer = Timer.periodic(_interval, (_) {
      final data = Int8List.fromList([
        throttle.clamp(-100, 100),
        steering.clamp(-100, 100),
      ]).buffer.asUint8List();
      _socket?.send(data, InternetAddress(host), port);
    });
  }

  void dispose() {
    _timer?.cancel();
    _socket?.close();
  }
}
