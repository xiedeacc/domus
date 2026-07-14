import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../data/map_repository.dart';

class GlobalMapPage extends ConsumerWidget {
  const GlobalMapPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final markers = ref.watch(mapMarkersProvider);
    return Scaffold(
      appBar: AppBar(title: const Text('Map')),
      body: markers.when(
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (error, _) => Center(child: Text('$error')),
        data: (markers) => markers.isEmpty
            ? const Center(child: Text('No map markers'))
            : LayoutBuilder(
                builder: (context, constraints) => Stack(
                  fit: StackFit.expand,
                  children: [
                    CustomPaint(painter: _WorldMapPainter()),
                    for (final marker in markers)
                      Positioned(
                        left:
                            ((marker.lon + 180) / 360) * constraints.maxWidth -
                            18,
                        top:
                            ((90 - marker.lat) / 180) * constraints.maxHeight -
                            18,
                        child: IconButton.filledTonal(
                          tooltip: [marker.city, marker.country]
                              .whereType<String>()
                              .where((value) => value.isNotEmpty)
                              .join(', '),
                          icon: const Icon(Icons.place),
                          onPressed: () =>
                              context.push('/asset/${marker.assetId}'),
                        ),
                      ),
                  ],
                ),
              ),
      ),
    );
  }
}

class _WorldMapPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final background = Paint()..color = const Color(0xffeef2f3);
    final grid = Paint()
      ..color = const Color(0xffb7c1c6)
      ..strokeWidth = 1;
    final equator = Paint()
      ..color = const Color(0xff6b8793)
      ..strokeWidth = 1.5;
    canvas.drawRect(Offset.zero & size, background);
    for (var i = 1; i < 6; i++) {
      final x = size.width * i / 6;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), grid);
    }
    for (var i = 1; i < 4; i++) {
      final y = size.height * i / 4;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), grid);
    }
    canvas.drawLine(
      Offset(0, size.height / 2),
      Offset(size.width, size.height / 2),
      equator,
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
