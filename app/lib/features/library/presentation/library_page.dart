import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

import '../../shared/immich_style.dart';

class LibraryPage extends StatelessWidget {
  const LibraryPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 28),
          children: [
            ImmichLogoHeader(
              actions: [
                ImmichRoundedIconButton(
                  icon: Icons.sync,
                  tooltip: '同步',
                  onPressed: () {},
                ),
                ImmichRoundedIconButton(
                  icon: Icons.close,
                  tooltip: '关闭',
                  filled: true,
                  onPressed: () {},
                ),
              ],
            ),
            const SizedBox(height: 4),
            GridView.count(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              crossAxisCount: 2,
              mainAxisSpacing: 12,
              crossAxisSpacing: 12,
              childAspectRatio: 3.15,
              children: const [
                ImmichQuickTile(
                  icon: Icons.favorite_border,
                  label: 'Favorites',
                ),
                ImmichQuickTile(
                  icon: Icons.archive_outlined,
                  label: 'Archived',
                ),
                ImmichQuickTile(icon: Icons.link, label: 'Shared links'),
                ImmichQuickTile(icon: Icons.delete_outline, label: 'Trash'),
              ],
            ),
            const SizedBox(height: 18),
            GridView.count(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              crossAxisCount: 2,
              mainAxisSpacing: 18,
              crossAxisSpacing: 18,
              childAspectRatio: 1.02,
              children: const [
                _DiscoveryCard(
                  icon: Icons.groups_outlined,
                  title: 'People',
                  accent: Color(0xFFE7E2F8),
                ),
                _DiscoveryCard(
                  icon: Icons.map_outlined,
                  title: 'Places',
                  accent: Color(0xFFCFE9EF),
                ),
              ],
            ),
            const ImmichSectionTitle('On this device'),
            _DevicePreviewCard(onTap: () => context.push('/folders')),
            const SizedBox(height: 12),
            _LibraryListGroup(
              children: [
                _LibraryListItem(
                  icon: Icons.folder_outlined,
                  label: 'Folders',
                  onTap: () => context.push('/folders'),
                ),
                const _LibraryListItem(
                  icon: Icons.lock_outline,
                  label: 'Locked Folder',
                ),
              ],
            ),
            const SizedBox(height: 14),
            _LibraryListGroup(
              children: [
                _LibraryListItem(
                  icon: Icons.backup_outlined,
                  label: 'Backup',
                  onTap: () => context.push('/backup'),
                ),
                _LibraryListItem(
                  icon: Icons.settings_outlined,
                  label: 'Settings',
                  onTap: () => context.push('/settings'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}

class _DiscoveryCard extends StatelessWidget {
  const _DiscoveryCard({
    required this.icon,
    required this.title,
    required this.accent,
  });

  final IconData icon;
  final String title;
  final Color accent;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Expanded(
          child: DecoratedBox(
            decoration: BoxDecoration(
              color: accent,
              borderRadius: BorderRadius.circular(22),
            ),
            child: Center(child: Icon(icon, size: 48, color: colors.primary)),
          ),
        ),
        const SizedBox(height: 12),
        Text(
          title,
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
          style: Theme.of(context).textTheme.titleLarge?.copyWith(
            color: immichPrimaryText,
            fontSize: 16,
            fontWeight: FontWeight.w700,
            letterSpacing: 0,
          ),
        ),
      ],
    );
  }
}

class _DevicePreviewCard extends StatelessWidget {
  const _DevicePreviewCard({required this.onTap});

  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return InkWell(
      borderRadius: BorderRadius.circular(22),
      onTap: onTap,
      child: AspectRatio(
        aspectRatio: 2.15,
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: colors.primary.withValues(alpha: 0.06),
            borderRadius: BorderRadius.circular(22),
          ),
          child: Padding(
            padding: const EdgeInsets.all(14),
            child: GridView.count(
              physics: const NeverScrollableScrollPhysics(),
              crossAxisCount: 2,
              mainAxisSpacing: 8,
              crossAxisSpacing: 8,
              children: [
                _MiniPreviewTile(color: colors.primary.withValues(alpha: 0.18)),
                _MiniPreviewTile(
                  color: colors.tertiary.withValues(alpha: 0.18),
                ),
                _MiniPreviewTile(
                  color: colors.secondary.withValues(alpha: 0.18),
                ),
                _MiniPreviewTile(color: colors.primary.withValues(alpha: 0.10)),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _MiniPreviewTile extends StatelessWidget {
  const _MiniPreviewTile({required this.color});

  final Color color;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: color,
        borderRadius: BorderRadius.circular(18),
      ),
      child: Icon(
        Icons.photo_size_select_actual_outlined,
        color: Theme.of(context).colorScheme.primary,
      ),
    );
  }
}

class _LibraryListGroup extends StatelessWidget {
  const _LibraryListGroup({required this.children});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.primary.withValues(alpha: 0.045),
        borderRadius: BorderRadius.circular(24),
      ),
      child: Column(
        children: [
          for (var i = 0; i < children.length; i++) ...[
            children[i],
            if (i != children.length - 1) const Divider(height: 1, indent: 72),
          ],
        ],
      ),
    );
  }
}

class _LibraryListItem extends StatelessWidget {
  const _LibraryListItem({required this.icon, required this.label, this.onTap});

  final IconData icon;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return InkWell(
      onTap: onTap,
      child: SizedBox(
        height: 62,
        child: Row(
          children: [
            const SizedBox(width: 20),
            Icon(icon, size: 24, color: immichPrimaryText),
            const SizedBox(width: 20),
            Expanded(
              child: Text(
                label,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: Theme.of(context).textTheme.titleLarge?.copyWith(
                  color: immichPrimaryText,
                  fontSize: 17,
                  fontWeight: FontWeight.w600,
                  letterSpacing: 0,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
