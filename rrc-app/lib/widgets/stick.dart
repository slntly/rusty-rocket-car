import 'package:material_ui/material_ui.dart';

/// A slider styled as a control stick.
///
/// The value is in the range [-100, 100] and resets to 0 when the pointer
/// is released.
class Stick extends StatelessWidget {
  const Stick({super.key, required this.value, required this.onChanged});

  final int value;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final cs = Theme.of(context).colorScheme;
    final markColor = cs.outlineVariant;

    return SliderTheme(
      data: SliderTheme.of(context).copyWith(
        activeTrackColor: cs.surfaceContainerHighest,
        inactiveTrackColor: cs.surfaceContainerHighest,
      ),
      child: Listener(
        onPointerUp: (_) => onChanged(0),
        onPointerCancel: (_) => onChanged(0),
        child: Stack(
          alignment: Alignment.center,
          children: [
            Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(width: 2, height: 10, color: markColor),
                const SizedBox(height: 52),
                Container(width: 2, height: 10, color: markColor),
              ],
            ),
            Slider(
              value: value.toDouble(),
              min: -100,
              max: 100,
              onChanged: (v) => onChanged(v.round()),
            ),
          ],
        ),
      ),
    );
  }
}
