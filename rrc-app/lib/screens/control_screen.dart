import 'package:material_ui/material_ui.dart';

import '../services/udp_sender.dart';
import '../widgets/stick.dart';

class ControlScreen extends StatefulWidget {
  const ControlScreen({super.key});

  @override
  State<ControlScreen> createState() => _ControlScreenState();
}

class _ControlScreenState extends State<ControlScreen> {
  final _udp = UdpSender();

  @override
  void initState() {
    super.initState();
    _udp.start();
  }

  @override
  void dispose() {
    _udp.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const len = 280.0;
    const hitBox = 160.0;

    return Scaffold(
      body: SafeArea(
        child: Align(
          alignment: const Alignment(0.0, 0.25),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 48),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              crossAxisAlignment: CrossAxisAlignment.center,
              children: [
                SizedBox(
                  width: len,
                  height: hitBox,
                  child: Stick(
                    value: _udp.steering,
                    onChanged: (v) => setState(() => _udp.steering = v),
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.only(right: (len - hitBox) * 0.5),
                  child: SizedBox(
                    width: hitBox,
                    height: len,
                    child: RotatedBox(
                      quarterTurns: -1,
                      child: Stick(
                        value: _udp.throttle,
                        onChanged: (v) => setState(() => _udp.throttle = v),
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
