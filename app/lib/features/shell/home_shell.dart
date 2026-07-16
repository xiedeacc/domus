import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../shared/immich_style.dart';

/// Adaptive navigation shell: bottom bar on narrow screens (phone), side
/// rail on wide screens (web / tablet / desktop).
class HomeShell extends StatelessWidget {
  const HomeShell({super.key, required this.shell});

  final StatefulNavigationShell shell;

  static const _destinations = [
    (
      icon: Icons.photo_library_outlined,
      selectedIcon: Icons.photo_library,
      label: '照片',
    ),
    (icon: Icons.search, selectedIcon: Icons.search, label: '搜索'),
    (
      icon: Icons.photo_album_outlined,
      selectedIcon: Icons.photo_album,
      label: '相簿',
    ),
    (
      icon: Icons.grid_view_outlined,
      selectedIcon: Icons.grid_view,
      label: '资源库',
    ),
  ];

  @override
  Widget build(BuildContext context) {
    final wide = MediaQuery.sizeOf(context).width >= 600;

    if (wide) {
      return Scaffold(
        body: Row(
          children: [
            NavigationRail(
              selectedIndex: shell.currentIndex,
              onDestinationSelected: shell.goBranch,
              labelType: NavigationRailLabelType.all,
              destinations: [
                for (final d in _destinations)
                  NavigationRailDestination(
                    icon: Icon(d.icon),
                    selectedIcon: Icon(d.selectedIcon),
                    label: Text(d.label),
                  ),
              ],
            ),
            const VerticalDivider(width: 1),
            Expanded(child: shell),
          ],
        ),
      );
    }

    return Scaffold(
      body: shell,
      bottomNavigationBar: _ImmichBottomNavigationBar(
        selectedIndex: shell.currentIndex,
        onDestinationSelected: shell.goBranch,
      ),
    );
  }
}

class _ImmichBottomNavigationBar extends StatelessWidget {
  const _ImmichBottomNavigationBar({
    required this.selectedIndex,
    required this.onDestinationSelected,
  });

  final int selectedIndex;
  final ValueChanged<int> onDestinationSelected;

  @override
  Widget build(BuildContext context) {
    final bottomInset = MediaQuery.paddingOf(context).bottom;
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.surface,
        border: Border(top: BorderSide(color: colors.outlineVariant)),
      ),
      child: SafeArea(
        top: false,
        child: SizedBox(
          height: 84 + bottomInset.clamp(0, 8),
          child: Row(
            children: [
              for (var i = 0; i < HomeShell._destinations.length; i++)
                Expanded(
                  child: _ImmichNavItem(
                    icon: HomeShell._destinations[i].icon,
                    selectedIcon: HomeShell._destinations[i].selectedIcon,
                    label: HomeShell._destinations[i].label,
                    selected: selectedIndex == i,
                    onTap: () => onDestinationSelected(i),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }
}

class _ImmichNavItem extends StatelessWidget {
  const _ImmichNavItem({
    required this.icon,
    required this.selectedIcon,
    required this.label,
    required this.selected,
    required this.onTap,
  });

  final IconData icon;
  final IconData selectedIcon;
  final String label;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    final active = colors.primary;
    final inactive = colors.onSurfaceVariant;
    return Semantics(
      selected: selected,
      button: true,
      label: label,
      child: InkResponse(
        onTap: onTap,
        radius: 44,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            AnimatedContainer(
              duration: const Duration(milliseconds: 160),
              curve: Curves.easeOut,
              width: 72,
              height: 36,
              decoration: BoxDecoration(
                color: selected
                    ? active.withValues(alpha: 0.14)
                    : Colors.transparent,
                borderRadius: BorderRadius.circular(24),
              ),
              child: Icon(
                selected ? selectedIcon : icon,
                color: selected ? active : inactive,
                size: 28,
              ),
            ),
            const SizedBox(height: 2),
            Text(
              label,
              maxLines: 1,
              style: Theme.of(context).textTheme.labelLarge?.copyWith(
                color: selected ? immichPrimaryText : inactive,
                fontSize: 14,
                fontWeight: selected ? FontWeight.w700 : FontWeight.w600,
                height: 1.0,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
