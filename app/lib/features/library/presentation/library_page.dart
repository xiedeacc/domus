import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class LibraryPage extends StatelessWidget {
  const LibraryPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('资源库')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
        children: [
          const _SectionTitle('资料库'),
          const _LibraryGroup(
            children: [
              _LibraryTile(icon: Icons.favorite_border, label: '收藏'),
              _LibraryTile(icon: Icons.archive_outlined, label: '归档'),
              _LibraryTile(icon: Icons.link, label: '共享链接'),
              _LibraryTile(icon: Icons.delete_outline, label: '回收站'),
            ],
          ),
          const _SectionTitle('发现'),
          const _LibraryGroup(
            children: [
              _LibraryTile(icon: Icons.groups_outlined, label: '人物'),
              _LibraryTile(icon: Icons.place_outlined, label: '地点'),
            ],
          ),
          const _SectionTitle('此设备'),
          _LibraryGroup(
            children: [
              _LibraryTile(
                icon: Icons.folder_outlined,
                label: '文件夹',
                onTap: () => context.push('/folders'),
              ),
              const _LibraryTile(icon: Icons.lock_outline, label: '锁定文件夹'),
            ],
          ),
          const SizedBox(height: 12),
          _LibraryGroup(
            children: [
              _LibraryTile(
                icon: Icons.backup_outlined,
                label: '备份',
                onTap: () => context.push('/backup'),
              ),
              _LibraryTile(
                icon: Icons.settings_outlined,
                label: '设置',
                onTap: () => context.push('/settings'),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 18, 16, 6),
      child: Text(
        text,
        style: Theme.of(context).textTheme.labelLarge?.copyWith(
          color: Theme.of(context).colorScheme.onSurfaceVariant,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class _LibraryGroup extends StatelessWidget {
  const _LibraryGroup({required this.children});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final colors = Theme.of(context).colorScheme;
    return DecoratedBox(
      decoration: BoxDecoration(
        color: colors.surfaceContainerLowest,
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: colors.outlineVariant),
      ),
      child: Column(
        children: [
          for (var i = 0; i < children.length; i++) ...[
            children[i],
            if (i != children.length - 1) const Divider(height: 1, indent: 56),
          ],
        ],
      ),
    );
  }
}

class _LibraryTile extends StatelessWidget {
  const _LibraryTile({required this.icon, required this.label, this.onTap});

  final IconData icon;
  final String label;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      dense: true,
      minLeadingWidth: 24,
      leading: Icon(icon, size: 24),
      title: Text(
        label,
        style: Theme.of(context).textTheme.titleMedium?.copyWith(fontSize: 17),
      ),
      trailing: const Icon(Icons.chevron_right),
      onTap: onTap,
    );
  }
}
