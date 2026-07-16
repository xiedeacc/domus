import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

const _pageColor = Color(0xFFFBFAFF);
const _panelColor = Color(0xFFF0EEF8);
const _primary = Color(0xFF4B55A8);
const _text = Color(0xFF202124);

class LibraryPage extends StatelessWidget {
  const LibraryPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: _pageColor,
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(18, 14, 18, 24),
          children: const [
            _LibraryTopBar(),
            SizedBox(height: 18),
            _ActionGrid(),
            SizedBox(height: 28),
            _DiscoveryGrid(),
            SizedBox(height: 22),
            Text(
              'On this device',
              style: TextStyle(
                fontSize: 27,
                fontWeight: FontWeight.w700,
                color: _text,
              ),
            ),
            SizedBox(height: 18),
            _DevicePanel(),
          ],
        ),
      ),
    );
  }
}

class _LibraryTopBar extends StatelessWidget {
  const _LibraryTopBar();

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Container(
          width: 38,
          height: 38,
          decoration: BoxDecoration(
            color: const Color(0xFFFF8B3D),
            borderRadius: BorderRadius.circular(11),
          ),
          child: const Icon(Icons.home_rounded, color: Colors.white, size: 26),
        ),
        const SizedBox(width: 8),
        const Text(
          'domus',
          style: TextStyle(
            color: _primary,
            fontSize: 32,
            fontWeight: FontWeight.w700,
          ),
        ),
        const Spacer(),
        IconButton(
          tooltip: 'Sync',
          onPressed: () {},
          icon: const Icon(Icons.sync, color: _primary, size: 30),
        ),
        const SizedBox(width: 4),
        const CircleAvatar(
          radius: 28,
          backgroundColor: Color(0xFFE8BE21),
          child: Text(
            'X',
            style: TextStyle(
              color: Colors.white,
              fontSize: 20,
              fontWeight: FontWeight.w700,
            ),
          ),
        ),
      ],
    );
  }
}

class _ActionGrid extends StatelessWidget {
  const _ActionGrid();

  @override
  Widget build(BuildContext context) {
    return GridView.count(
      crossAxisCount: 2,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      mainAxisSpacing: 12,
      crossAxisSpacing: 12,
      childAspectRatio: 2.9,
      children: const [
        _ActionTile(icon: Icons.favorite_border, label: 'Favorites'),
        _ActionTile(icon: Icons.archive_outlined, label: 'Archived'),
        _ActionTile(
          icon: Icons.link,
          label: 'Shared Links',
          route: '/settings',
        ),
        _ActionTile(icon: Icons.delete_outline, label: 'Trash'),
      ],
    );
  }
}

class _ActionTile extends StatelessWidget {
  const _ActionTile({required this.icon, required this.label, this.route});

  final IconData icon;
  final String label;
  final String? route;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: route == null ? null : () => context.push(route!),
      borderRadius: BorderRadius.circular(28),
      child: Container(
        decoration: BoxDecoration(
          color: _pageColor,
          borderRadius: BorderRadius.circular(28),
          border: Border.all(color: const Color(0xFFE5E2EA)),
        ),
        padding: const EdgeInsets.symmetric(horizontal: 18),
        child: Row(
          children: [
            Icon(icon, color: _primary, size: 27),
            const SizedBox(width: 14),
            Expanded(
              child: FittedBox(
                fit: BoxFit.scaleDown,
                alignment: Alignment.centerLeft,
                child: Text(
                  label,
                  maxLines: 1,
                  style: const TextStyle(
                    fontSize: 22,
                    fontWeight: FontWeight.w700,
                    color: _text,
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DiscoveryGrid extends StatelessWidget {
  const _DiscoveryGrid();

  @override
  Widget build(BuildContext context) {
    return GridView.count(
      crossAxisCount: 2,
      shrinkWrap: true,
      physics: const NeverScrollableScrollPhysics(),
      mainAxisSpacing: 22,
      crossAxisSpacing: 14,
      childAspectRatio: 0.86,
      children: const [
        _DiscoveryTile(
          label: 'People',
          child: Icon(Icons.groups_outlined, color: _primary, size: 72),
        ),
        _DiscoveryTile(label: 'Places', child: _MapPreview()),
      ],
    );
  }
}

class _DiscoveryTile extends StatelessWidget {
  const _DiscoveryTile({required this.label, required this.child});

  final String label;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: ClipRRect(
            borderRadius: BorderRadius.circular(18),
            child: Container(
              width: double.infinity,
              color: _panelColor,
              child: child,
            ),
          ),
        ),
        const SizedBox(height: 12),
        Text(
          label,
          style: const TextStyle(
            fontSize: 24,
            fontWeight: FontWeight.w700,
            color: _text,
          ),
        ),
      ],
    );
  }
}

class _MapPreview extends StatelessWidget {
  const _MapPreview();

  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: _MapPreviewPainter(),
      child: const Center(
        child: Text(
          'Honolulu',
          style: TextStyle(
            fontSize: 26,
            fontWeight: FontWeight.w700,
            color: Color(0xFF67616A),
          ),
        ),
      ),
    );
  }
}

class _MapPreviewPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final water = Paint()..color = const Color(0xFFA9D5DF);
    final land = Paint()..color = const Color(0xFFE7E3D5);
    final road = Paint()
      ..color = Colors.white
      ..strokeWidth = 3
      ..style = PaintingStyle.stroke;

    canvas.drawRect(Offset.zero & size, water);
    final path = Path()
      ..moveTo(size.width * 0.08, size.height * 0.18)
      ..cubicTo(
        size.width * 0.42,
        size.height * 0.02,
        size.width * 0.72,
        size.height * 0.26,
        size.width * 0.92,
        size.height * 0.18,
      )
      ..lineTo(size.width, size.height)
      ..lineTo(0, size.height)
      ..close();
    canvas.drawPath(path, land);

    final roadPath = Path()
      ..moveTo(size.width * 0.12, size.height * 0.72)
      ..cubicTo(
        size.width * 0.36,
        size.height * 0.54,
        size.width * 0.58,
        size.height * 0.56,
        size.width * 0.88,
        size.height * 0.38,
      );
    canvas.drawPath(roadPath, road);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

class _DevicePanel extends StatelessWidget {
  const _DevicePanel();

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        color: _panelColor,
        borderRadius: BorderRadius.circular(28),
        border: Border.all(color: const Color(0xFFE7E3EE)),
      ),
      child: const Column(
        children: [
          _DeviceRow(
            icon: Icons.folder_outlined,
            label: 'Folders',
            route: '/folders',
            showDivider: true,
          ),
          _DeviceRow(icon: Icons.lock_outline, label: 'Locked Folder'),
        ],
      ),
    );
  }
}

class _DeviceRow extends StatelessWidget {
  const _DeviceRow({
    required this.icon,
    required this.label,
    this.route,
    this.showDivider = false,
  });

  final IconData icon;
  final String label;
  final String? route;
  final bool showDivider;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: route == null ? null : () => context.push(route!),
      borderRadius: BorderRadius.circular(28),
      child: Container(
        height: 92,
        decoration: BoxDecoration(
          border: showDivider
              ? const Border(bottom: BorderSide(color: Color(0xFFE7E3EE)))
              : null,
        ),
        padding: const EdgeInsets.symmetric(horizontal: 26),
        child: Row(
          children: [
            Icon(icon, color: const Color(0xFF55525D), size: 32),
            const SizedBox(width: 28),
            Text(
              label,
              style: const TextStyle(
                fontSize: 24,
                fontWeight: FontWeight.w700,
                color: _text,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
