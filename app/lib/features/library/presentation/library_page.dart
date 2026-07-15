import 'package:flutter/material.dart';
import 'package:go_router/go_router.dart';

class LibraryPage extends StatelessWidget {
  const LibraryPage({super.key});

  @override
  Widget build(BuildContext context) {
    final items = [
      _LibraryItem(
        icon: Icons.cloud_upload_outlined,
        title: '备份',
        subtitle: '自动备份、相册选择和后台上传',
        route: '/backup',
      ),
      _LibraryItem(
        icon: Icons.folder_outlined,
        title: '文件夹',
        subtitle: '按服务端文件夹浏览资源',
        route: '/folders',
      ),
      _LibraryItem(
        icon: Icons.map_outlined,
        title: '地图',
        subtitle: '按位置信息浏览照片',
        route: '/map',
      ),
      _LibraryItem(
        icon: Icons.auto_awesome_outlined,
        title: '回忆',
        subtitle: '按年份和时间线生成回忆',
        route: '/memories',
      ),
      _LibraryItem(
        icon: Icons.settings_outlined,
        title: '设置',
        subtitle: '服务器、系统配置和共享链接',
        route: '/settings',
      ),
    ];

    return Scaffold(
      appBar: AppBar(title: const Text('资源库')),
      body: ListView.separated(
        padding: const EdgeInsets.fromLTRB(16, 12, 16, 24),
        itemBuilder: (context, index) {
          final item = items[index];
          return ListTile(
            leading: Icon(item.icon),
            title: Text(item.title),
            subtitle: Text(item.subtitle),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => context.push(item.route),
          );
        },
        separatorBuilder: (_, _) => const Divider(height: 1),
        itemCount: items.length,
      ),
    );
  }
}

class _LibraryItem {
  const _LibraryItem({
    required this.icon,
    required this.title,
    required this.subtitle,
    required this.route,
  });

  final IconData icon;
  final String title;
  final String subtitle;
  final String route;
}
